use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;
use rusqlite::params;
use serde::Deserialize;

use crate::utilities::database::init::Database;

#[derive(Deserialize)]
struct Settings {
    base: Base,
    flaresolverr: Flaresolverr,
    wordpress_api: WordPressApi,
    wordpress_page: WordPressPage,
}

#[derive(Deserialize)]
struct Base {
    name: String,
    version: String,
    max_concurrency: i32,
}

#[derive(Deserialize)]
struct Flaresolverr {
    flaresolverr_url: String,
}

#[derive(Deserialize)]
struct WordPressApi {
    wordpress_url: String,
    username_api: String,
    password_api: String,
}

#[derive(Deserialize)]
struct WordPressPage {
    template: String,
    status: String,
    parent: i32,
    author: i32,
}

pub fn load_configuration(db: &Database, file_path: &str) -> Result<()> {
    // Read the settings file
    let content = fs::read_to_string(file_path).context("Failed to read settings file")?;

    // Parse the settings file
    let settings: Settings = toml::from_str(&content).context("Failed to parse settings file")?;

    let conn = db.conn.lock().unwrap();
    // Insert settings into the configuration table
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["base_name", settings.base.name],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["base_version", settings.base.version],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["base_max_concurrency", settings.base.max_concurrency.to_string()],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["flaresolverr_url", settings.flaresolverr.flaresolverr_url],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["wordpress_url", settings.wordpress_api.wordpress_url],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["username_api", settings.wordpress_api.username_api],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["password_api", settings.wordpress_api.password_api],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["wordpress_template", settings.wordpress_page.template],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["wordpress_status", settings.wordpress_page.status],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["wordpress_parent", settings.wordpress_page.parent.to_string()],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["wordpress_author", settings.wordpress_page.author.to_string()],
    )?;

    println!("{}", "Settings loaded and inserted into the database".green());

    Ok(())
}