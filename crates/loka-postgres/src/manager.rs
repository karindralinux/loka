use std::sync::Arc;

use dashmap::DashMap;
use loka_core::{
    ConnectionConfig, ConnectionId, GridPage, GridPageRequest, GridUpdateRequest, LokaError,
    Result, SqlColumn, SqlQueryResult, TableId,
};
use serde_json::Value as Json;
use tokio_postgres::{Client, NoTls};

pub struct ConnectionManager {
    clients: DashMap<ConnectionId, Arc<Client>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    fn build_config(cfg: &ConnectionConfig) -> tokio_postgres::Config {
        let mut pg_cfg = tokio_postgres::Config::new();
        pg_cfg.host(&cfg.host);
        pg_cfg.port(cfg.port);
        pg_cfg.user(&cfg.username);
        if !cfg.password.is_empty() {
            pg_cfg.password(&cfg.password);
        }
        pg_cfg.dbname(&cfg.database);
        // MVP: ssl_mode ignored, NoTls used
        pg_cfg
    }

    pub async fn test_connection(cfg: &ConnectionConfig) -> Result<()> {
        let pg_cfg = Self::build_config(cfg);
        println!("[POSTGRES] Testing connection: host={} dbname={}", cfg.host, cfg.database);
        let (_client, connection) = pg_cfg.connect(NoTls)
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        // drive connection briefly
        tokio::spawn(async move {
            let _ = connection.await;
        });

        Ok(())
    }

    pub async fn connect(&self, cfg: &ConnectionConfig) -> Result<ConnectionId> {
        let pg_cfg = Self::build_config(cfg);
        println!("[POSTGRES] Connecting: host={} dbname={}", cfg.host, cfg.database);

        let (client, connection) = pg_cfg.connect(NoTls)
            .await
            .map_err(|e| {
                eprintln!("[POSTGRES] Connection FAILED: {}", e);
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("[POSTGRES] Connection Error: {}", e);
            }
        });

        let id = ConnectionId::new();
        println!("[POSTGRES] Connected! Runtime ID: {}", id.0);
        self.clients.insert(id, Arc::new(client));
        Ok(id)
    }

    pub fn get(&self, id: &ConnectionId) -> Result<Arc<Client>> {
        self.clients
            .get(id)
            .map(|r| r.value().clone())
            .ok_or_else(|| LokaError::ConnectionNotFound(id.0.to_string()))
    }

    pub fn disconnect(&self, id: &ConnectionId) -> Result<()> {
        self.clients.remove(id);
        Ok(())
    }

    pub async fn execute_sql(&self, id: &ConnectionId, sql: &str) -> Result<SqlQueryResult> {
        println!("[POSTGRES] Execute SQL (id={}): {}", id.0, sql);
        let client = self.get(id)?;

        let stmt = client
            .prepare(sql)
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        let columns = stmt
            .columns()
            .iter()
            .map(|c| SqlColumn {
                name: c.name().to_string(),
                type_name: c.type_().name().to_string(),
            })
            .collect::<Vec<_>>();

        let rows_pg = client
            .query(&stmt, &[])
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        // MVP: convert each cell to a string-ish JSON value.
        // Later we can add proper type decoding (numbers/bools/json/timestamps/etc).
        let mut rows: Vec<Vec<Json>> = Vec::with_capacity(rows_pg.len());

        for r in rows_pg.iter() {
            let mut out_row: Vec<Json> = Vec::with_capacity(columns.len());
            for (idx, col) in stmt.columns().iter().enumerate() {
                let type_name = col.type_().name();
                out_row.push(Self::cell_to_json(r, idx, type_name));
            }
            rows.push(out_row);
        }

        Ok(SqlQueryResult {
            row_count: rows.len(),
            columns,
            rows,
        })
    }

    fn cell_to_json(row: &tokio_postgres::Row, idx: usize, type_name: &str) -> Json {
        // Handle NULL for all types
        // We'll attempt Option<T> first; if it's NULL we return Json::Null.
        match type_name {
            "bool" => row
                .try_get::<usize, Option<bool>>(idx)
                .ok()
                .flatten()
                .map(Json::Bool)
                .unwrap_or(Json::Null),

            "int2" => row
                .try_get::<usize, Option<i16>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::Number(v.into()))
                .unwrap_or(Json::Null),

            "int4" => row
                .try_get::<usize, Option<i32>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::Number(v.into()))
                .unwrap_or(Json::Null),

            "int8" => row
                .try_get::<usize, Option<i64>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::Number(v.into()))
                .unwrap_or(Json::Null),

            "float4" => row
                .try_get::<usize, Option<f32>>(idx)
                .ok()
                .flatten()
                .and_then(|v| serde_json::Number::from_f64(v as f64))
                .map(Json::Number)
                .unwrap_or(Json::Null),

            "float8" => row
                .try_get::<usize, Option<f64>>(idx)
                .ok()
                .flatten()
                .and_then(serde_json::Number::from_f64)
                .map(Json::Number)
                .unwrap_or(Json::Null),

            "json" | "jsonb" => {
                let s = row.try_get::<usize, Option<String>>(idx).ok().flatten();
                match s {
                    None => Json::Null,
                    Some(s) => serde_json::from_str(&s).unwrap_or(Json::String(s)),
                }
            }

            "uuid" => row
                .try_get::<usize, Option<uuid::Uuid>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::String(v.to_string()))
                .unwrap_or(Json::Null),

            "timestamp" => row
                .try_get::<usize, Option<chrono::NaiveDateTime>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::String(v.to_string()))
                .unwrap_or(Json::Null),

            "timestamptz" => row
                .try_get::<usize, Option<chrono::DateTime<chrono::Utc>>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::String(v.to_string()))
                .unwrap_or(Json::Null),

            "date" => row
                .try_get::<usize, Option<chrono::NaiveDate>>(idx)
                .ok()
                .flatten()
                .map(|v| Json::String(v.to_string()))
                .unwrap_or(Json::Null),

            // default: treat as string
            _ => row
                .try_get::<usize, Option<String>>(idx)
                .ok()
                .flatten()
                .map(Json::String)
                .unwrap_or(Json::Null),
        }
    }

    pub async fn list_tables(&self, id: &ConnectionId, schema: &str) -> Result<Vec<String>> {
        println!("[POSTGRES] Listing tables in schema: {}", schema);
        let client = self.get(id)?;

        let rows = client
            .query(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name",
                &[&schema],
            )
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        // print this query
        println!(
            "[POSTGRES] Found tables in {}: {:?}",
            schema,
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name"
        );

        let tables: Vec<String> = rows
            .into_iter()
            .filter_map(|r| r.try_get::<_, String>(0).ok())
            .collect();
        println!("[POSTGRES] Found tables in {}: {:?}", schema, tables);
        Ok(tables)
    }

    pub async fn pk_columns(&self, id: &ConnectionId, table: &TableId) -> Result<Vec<String>> {
        let client = self.get(id)?;

        let rows = client
            .query(
                r#"
            select kcu.column_name
            from information_schema.table_constraints tc
            join information_schema.key_column_usage kcu
              on tc.constraint_name = kcu.constraint_name
             and tc.table_schema = kcu.table_schema
            where tc.constraint_type = 'PRIMARY KEY'
              and tc.table_schema = $1
              and tc.table_name = $2
            order by kcu.ordinal_position
            "#,
                &[&table.schema, &table.name],
            )
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        let pks: Vec<String> = rows
            .into_iter()
            .filter_map(|r| r.try_get::<_, String>(0).ok())
            .collect();
        println!("[POSTGRES] Primary keys for {}.{}: {:?}", table.schema, table.name, pks);
        Ok(pks)
    }

    pub async fn grid_page(&self, id: &ConnectionId, req: &GridPageRequest) -> Result<GridPage> {
        let client = self.get(id)?;

        let pk_columns = self.pk_columns(id, &req.table).await?;

        let order_cols = if let Some(ob) = &req.order_by {
            ob.clone()
        } else if !pk_columns.is_empty() {
            pk_columns.clone()
        } else {
            Vec::new()
        };

        let order_by_sql = if order_cols.is_empty() {
            String::new()
        } else {
            let parts = order_cols
                .iter()
                .map(|c| format!(r#""{}""#, c.replace('"', r#""""#)))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" ORDER BY {parts}")
        };

        let sql = format!(
            r#"SELECT * FROM "{}"."{}"{} LIMIT $1 OFFSET $2"#,
            req.table.schema.replace('"', r#""""#),
            req.table.name.replace('"', r#""""#),
            order_by_sql
        );

        println!("[POSTGRES] Grid Page Fetch SQL: {}", sql);
        let stmt = client
            .prepare(&sql)
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        let columns = stmt
            .columns()
            .iter()
            .map(|c| SqlColumn {
                name: c.name().to_string(),
                type_name: c.type_().name().to_string(),
            })
            .collect::<Vec<_>>();

        let rows_pg = client
            .query(&stmt, &[&req.limit, &req.offset])
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        let mut rows = Vec::with_capacity(rows_pg.len());
        for r in rows_pg.iter() {
            let mut out_row = Vec::with_capacity(columns.len());
            for (idx, col) in stmt.columns().iter().enumerate() {
                out_row.push(Self::cell_to_json(r, idx, col.type_().name()));
            }
            rows.push(out_row);
        }

        Ok(GridPage {
            row_count: rows.len(),
            columns,
            pk_columns,
            rows,
        })
    }

    pub async fn grid_update(&self, id: &ConnectionId, req: &GridUpdateRequest) -> Result<u64> {
        let client = self.get(id)?;

        if req.pk.is_empty() {
            return Err(LokaError::NotSupported(
                "editable grid requires a primary key".into(),
            ));
        }

        // UPDATE "schema"."table" SET "col" = $1::text WHERE "pk1" = $2::text AND ...
        let mut where_parts = Vec::new();
        for (i, (col, _)) in req.pk.iter().enumerate() {
            let arg_i = i + 2;
            where_parts.push(format!(
                r#""{}"::text = ${}::text"#,
                col.replace('"', r#""""#),
                arg_i
            ));
        }

        let sql = format!(
            r#"UPDATE "{}"."{}" SET "{}" = $1::text WHERE {}"#,
            req.table.schema.replace('"', r#""""#),
            req.table.name.replace('"', r#""""#),
            req.column.replace('"', r#""""#),
            where_parts.join(" AND ")
        );

        // bind everything as text (MVP)
        let mut params: Vec<Option<String>> = Vec::with_capacity(1 + req.pk.len());
        let to_sql_val = |v: &serde_json::Value| match v {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => Some(s.clone()),
            _ => Some(v.to_string()),
        };

        params.push(to_sql_val(&req.value));

        for (_, v) in req.pk.iter() {
            params.push(to_sql_val(v));
        }

        let bind: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p as _).collect();

        println!("[POSTGRES] Grid Update SQL: {} (params: {:?})", sql, params);
        let n = client
            .execute(&sql, &bind)
            .await
            .map_err(|e| {
                let msg = e.as_db_error().map(|db| db.message().to_string()).unwrap_or_else(|| e.to_string());
                LokaError::Database(msg)
            })?;

        if n == 0 {
            return Err(LokaError::Database(
                "no rows were updated. ensure the primary key hasn't changed or been deleted."
                    .into(),
            ));
        }

        Ok(n)
    }
}
