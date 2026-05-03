mod connection_store;

use once_cell::sync::Lazy;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            connections_list,
            connections_get,
            connections_save,
            connections_update,
            connections_delete,
            connections_test,
            connections_connect,
            pg_test_connection,
            pg_connect,
            pg_execute_sql,
            pg_grid_page,
            pg_grid_update,
            pg_list_schemas,
            pg_list_tables,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn ping(name: String) -> String {
    format!("pong: {name}")
}

static PG: Lazy<loka_postgres::manager::ConnectionManager> =
    Lazy::new(loka_postgres::manager::ConnectionManager::new);

fn sanitize_connection(mut conn: loka_core::SavedConnection) -> loka_core::SavedConnection {
    conn.config.password = String::new();
    conn
}

#[tauri::command]
async fn connections_list(
    app: tauri::AppHandle,
) -> Result<Vec<loka_core::SavedConnection>, String> {
    let list = connection_store::list_connections(&app)?;
    Ok(list.into_iter().map(sanitize_connection).collect())
}

#[tauri::command]
async fn connections_get(
    app: tauri::AppHandle,
    id: loka_core::SavedConnectionId,
) -> Result<loka_core::SavedConnection, String> {
    let conn = connection_store::get_connection(&app, id)?;
    Ok(sanitize_connection(conn))
}

#[tauri::command]
async fn connections_save(
    app: tauri::AppHandle,
    input: loka_core::SavedConnectionInput,
) -> Result<loka_core::SavedConnection, String> {
    let conn = connection_store::save_connection(&app, input)?;
    Ok(sanitize_connection(conn))
}

#[tauri::command]
async fn connections_update(
    app: tauri::AppHandle,
    input: loka_core::SavedConnectionUpdate,
) -> Result<loka_core::SavedConnection, String> {
    let conn = connection_store::update_connection(&app, input)?;
    Ok(sanitize_connection(conn))
}

#[tauri::command]
async fn connections_delete(
    app: tauri::AppHandle,
    id: loka_core::SavedConnectionId,
) -> Result<bool, String> {
    connection_store::delete_connection(&app, id)
}

#[tauri::command]
async fn connections_test(
    app: tauri::AppHandle,
    input: loka_core::ConnectionConfig,
    id: Option<loka_core::SavedConnectionId>,
) -> Result<(), String> {
    let cfg = connection_store::merge_password_from_store(&app, input, id)?;
    loka_postgres::manager::ConnectionManager::test_connection(&cfg)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn connections_connect(
    app: tauri::AppHandle,
    id: loka_core::SavedConnectionId,
) -> Result<loka_core::ConnectResult, String> {
    let conn = connection_store::get_connection(&app, id)?;
    let runtime_id = PG.connect(&conn.config).await.map_err(|e| e.to_string())?;
    Ok(loka_core::ConnectResult {
        runtime_id: runtime_id.0.to_string(),
        capabilities: loka_core::EngineCapabilities {
            sql: true,
            introspection: true,
            editable_grid: true,
        },
    })
}

#[tauri::command]
async fn pg_list_tables(runtime_id: String, schema: String) -> Result<Vec<String>, String> {
    let id = loka_core::ConnectionId(
        runtime_id
            .parse()
            .map_err(|e| format!("invalid runtime_id: {e}"))?,
    );
    PG.list_tables(&id, &schema).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn pg_list_schemas(runtime_id: String) -> Result<Vec<String>, String> {
    let id = loka_core::ConnectionId(
        runtime_id
            .parse()
            .map_err(|e| format!("invalid runtime_id: {e}"))?,
    );
    PG.list_schemas(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn pg_test_connection(cfg: loka_core::ConnectionConfig) -> Result<(), String> {
    loka_postgres::manager::ConnectionManager::test_connection(&cfg)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn pg_connect(cfg: loka_core::ConnectionConfig) -> Result<loka_core::ConnectResult, String> {
    println!("[TAURI] pg_connect: {}@{}:{}", cfg.username, cfg.host, cfg.database);
    let conn_id = PG.connect(&cfg).await.map_err(|e| e.to_string())?;

    Ok(loka_core::ConnectResult {
        runtime_id: conn_id.0.to_string(),
        capabilities: loka_core::EngineCapabilities {
            sql: true,
            introspection: false,
            editable_grid: false,
        },
    })
}

#[tauri::command]
async fn pg_execute_sql(
    runtime_id: String,
    sql: String,
) -> Result<loka_core::SqlQueryResult, String> {
    println!("[TAURI] pg_execute_sql: {}", sql);
    let id = loka_core::ConnectionId(
        runtime_id
            .parse()
            .map_err(|e| format!("invalid runtime_id: {e}"))?,
    );

    PG.execute_sql(&id, &sql).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn pg_grid_page(
    runtime_id: String,
    req: loka_core::GridPageRequest,
) -> Result<loka_core::GridPage, String> {
    println!("[TAURI] pg_grid_page: {}.{}", req.table.schema, req.table.name);
    let id = loka_core::ConnectionId(
        runtime_id
            .parse()
            .map_err(|e| format!("invalid runtime_id: {e}"))?,
    );
    PG.grid_page(&id, &req).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn pg_grid_update(
    runtime_id: String,
    req: loka_core::GridUpdateRequest,
) -> Result<u64, String> {
    println!("[TAURI] pg_grid_update: {}.{}", req.table.schema, req.table.name);
    let id = loka_core::ConnectionId(
        runtime_id
            .parse()
            .map_err(|e| format!("invalid runtime_id: {e}"))?,
    );
    PG.grid_update(&id, &req).await.map_err(|e| e.to_string())
}
