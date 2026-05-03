use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EngineType {
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub sql: bool,
    pub introspection: bool,
    pub editable_grid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResult {
    pub runtime_id: String,
    pub capabilities: EngineCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub engine: EngineType,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: String, // keep as string for now ("disable", "prefer", etc)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlColumn {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<SqlColumn>,
    pub rows: Vec<Vec<Value>>, // JSON-friendly cell values
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableId {
    pub schema: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPageRequest {
    pub table: TableId,
    pub limit: i64,
    pub offset: i64,
    pub order_by: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPage {
    pub columns: Vec<SqlColumn>,
    pub pk_columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridUpdateRequest {
    pub table: TableId,
    pub pk: Vec<(String, serde_json::Value)>,
    pub column: String,
    pub value: serde_json::Value,
}