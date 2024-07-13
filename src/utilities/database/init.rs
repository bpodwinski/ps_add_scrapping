use std::env;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use rusqlite::Connection;

#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

/// Initializes the SQLite database.
///
/// This function performs the following steps:
///
/// 1. Retrieves the current executable path.
/// 2. Navigates to the directory containing the executable.
/// 3. Checks if the SQLite file `urls.sqlite` exists.
/// 4. Opens a connection to the SQLite database.
/// 5. Creates the "urls" table if the database file didn't exist before.
///
/// # Errors
///
/// This function will return an error if any of the following operations fail:
///
/// - Retrieving the current executable path.
/// - Opening a connection to the SQLite database.
/// - Creating the "urls" table if it didn't exist.
///
/// # Returns
///
/// If successful, returns an open connection to the SQLite database.
pub fn init() -> anyhow::Result<Database> {
    // Get the current executable path
    let mut db_path = env::current_exe().context("Failed to get current executable path")?;

    // Navigate to the correct directory
    db_path.pop();
    db_path.push("urls.sqlite");

    // Check if the SQLite file exists
    let db_exists = db_path.exists();

    // Open a connection to the SQLite database
    let conn = Connection::open(&db_path).context("Failed to open SQLite database")?;

    // Create the "urls" table if the database file didn't exist before
    if !db_exists {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                last_mod TEXT,
                change_freq TEXT,
                http_code INTEGER,
                date_modified TEXT
            )",
            [],
        ).context("Failed to create table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS configuration (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL
            )",
            [],
        ).context("Failed to create configuration table")?;
    }

    Ok(Database::new(Arc::new(Mutex::new(conn))))
}
