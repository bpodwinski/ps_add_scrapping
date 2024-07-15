use std::sync::Arc;

use anyhow::{Context, Result};
use rusqlite::Connection;
use tokio::sync::Mutex;

pub async fn get_configuration_value(conn: &Arc<Mutex<Connection>>, key: &str) -> Result<String> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT value FROM configuration WHERE key = ?1")?;
    let value: String = stmt
        .query_row([key], |row| row.get(0))
        .context(format!("Failed to get configuration for key: {}", key))?;
    Ok(value)
}

pub async fn get_configuration_value_as_usize(
    conn: &Arc<Mutex<Connection>>,
    key: &str,
) -> Result<usize> {
    let value = get_configuration_value(conn, key).await?;
    let parsed_value = value.parse::<usize>().context(format!(
        "Failed to parse configuration value as usize for key: {}",
        key
    ))?;
    Ok(parsed_value)
}

pub async fn get_configuration_value_as_i64(
    conn: &Arc<Mutex<Connection>>,
    key: &str,
) -> Result<i64> {
    let value = get_configuration_value(conn, key).await?;
    let parsed_value = value.parse::<i64>().context(format!(
        "Failed to parse configuration value as i64 for key: {}",
        key
    ))?;
    Ok(parsed_value)
}
