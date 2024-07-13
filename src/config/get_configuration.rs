use std::sync::Arc;

use anyhow::{Context, Result};
use rusqlite::Connection;
use tokio::sync::Mutex;

pub async fn get_configuration_value(conn: Arc<Mutex<Connection>>, key: &str) -> Result<String> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT value FROM configuration WHERE key = ?1")?;
    let value: String = stmt.query_row([key], |row| row.get(0))
        .context(format!("Failed to get configuration for key: {}", key))?;
    Ok(value)
}