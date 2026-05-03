use std::fs;
use std::path::PathBuf;

use loka_core::{ConnectionConfig, SavedConnection, SavedConnectionId, SavedConnectionInput, SavedConnectionUpdate};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConnectionStoreData {
    connections: Vec<SavedConnection>,
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir error: {e}"))?;
    fs::create_dir_all(&base).map_err(|e| format!("create app data dir failed: {e}"))?;
    Ok(base.join("connections.json"))
}

fn load_store(path: &PathBuf) -> Result<ConnectionStoreData, String> {
    if !path.exists() {
        return Ok(ConnectionStoreData::default());
    }
    let data = fs::read_to_string(path).map_err(|e| format!("read store failed: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("parse store failed: {e}"))
}

fn write_store(path: &PathBuf, data: &ConnectionStoreData) -> Result<(), String> {
    let tmp_path = path.with_extension("json.tmp");
    let payload = serde_json::to_string_pretty(data).map_err(|e| format!("serialize store failed: {e}"))?;
    fs::write(&tmp_path, payload).map_err(|e| format!("write store failed: {e}"))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("persist store failed: {e}"))?;
    Ok(())
}

pub fn list_connections(app: &AppHandle) -> Result<Vec<SavedConnection>, String> {
    let path = store_path(app)?;
    let data = load_store(&path)?;
    Ok(data.connections)
}

pub fn get_connection(app: &AppHandle, id: SavedConnectionId) -> Result<SavedConnection, String> {
    let path = store_path(app)?;
    let data = load_store(&path)?;
    data.connections
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| "connection not found".to_string())
}

pub fn save_connection(app: &AppHandle, input: SavedConnectionInput) -> Result<SavedConnection, String> {
    let path = store_path(app)?;
    let mut data = load_store(&path)?;

    let connection = SavedConnection {
        id: SavedConnectionId::new(),
        name: input.name,
        config: input.config,
    };

    data.connections.push(connection.clone());
    write_store(&path, &data)?;
    Ok(connection)
}

pub fn update_connection(app: &AppHandle, input: SavedConnectionUpdate) -> Result<SavedConnection, String> {
    let path = store_path(app)?;
    let mut data = load_store(&path)?;

    let mut updated = None;
    for conn in data.connections.iter_mut() {
        if conn.id == input.id {
            if input.config.password.is_empty() {
                let mut cfg = input.config.clone();
                cfg.password = conn.config.password.clone();
                conn.config = cfg;
            } else {
                conn.config = input.config.clone();
            }
            conn.name = input.name.clone();
            updated = Some(conn.clone());
            break;
        }
    }

    let Some(updated) = updated else {
        return Err("connection not found".to_string());
    };

    write_store(&path, &data)?;
    Ok(updated)
}

pub fn delete_connection(app: &AppHandle, id: SavedConnectionId) -> Result<bool, String> {
    let path = store_path(app)?;
    let mut data = load_store(&path)?;
    let before = data.connections.len();
    data.connections.retain(|c| c.id != id);
    let removed = data.connections.len() != before;
    if removed {
        write_store(&path, &data)?;
    }
    Ok(removed)
}

pub fn merge_password_from_store(
    app: &AppHandle,
    mut cfg: ConnectionConfig,
    id: Option<SavedConnectionId>,
) -> Result<ConnectionConfig, String> {
    if !cfg.password.is_empty() || id.is_none() {
        return Ok(cfg);
    }
    let saved = get_connection(app, id.unwrap())?;
    cfg.password = saved.config.password;
    Ok(cfg)
}
