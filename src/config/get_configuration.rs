use anyhow::{Context, Result};

use crate::utilities::database::init::Database;

pub fn get_configuration_value(db: &Database, key: &str) -> Result<String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT value FROM configuration WHERE key = ?1")?;
    let value: String = stmt.query_row([key], |row| row.get(0))
        .context(format!("Failed to get configuration for key: {}", key))?;
    Ok(value)
}
