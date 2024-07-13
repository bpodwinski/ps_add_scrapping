use std::fs;
use std::sync::Arc;

use anyhow::{Context, Result};
use colored::Colorize;
use rusqlite::{Connection, params};
use serde::Deserialize;
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct Settings {
    base: Base,
    processing: Processing,
    prestashop_addon: PrestashopAddon,
    flaresolverr: Flaresolverr,
    wordpress_api: WordPressApi,
    wordpress_page: WordPressPage,
}

#[derive(Deserialize)]
struct Base {
    app_name: String,
    app_version: String,
}

#[derive(Deserialize)]
struct Processing {
    batch_size: u32,
    max_concurrency: u32,
}

#[derive(Deserialize)]
struct PrestashopAddon {
    robots_url: String,
    sitemap_lang: String,
}

#[derive(Deserialize)]
struct Flaresolverr {
    flaresolverr_url: String,
    user_agent: String,
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
    parent: u32,
    author: u32,
}

pub async fn load_configuration(db: &Arc<Mutex<Connection>>, file_path: &str) -> Result<()> {
    let conn = db.lock().await;

    // Read the settings file
    let content = fs::read_to_string(file_path).context("Failed to read settings file")?;

    // Parse the settings file
    let settings: Settings = toml::from_str(&content).context("Failed to parse settings file")?;

    // Insert settings into the configuration table
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["app_name", settings.base.app_name],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["app_version", settings.base.app_version],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["batch_size", settings.processing.batch_size.to_string()],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["max_concurrency", settings.processing.max_concurrency.to_string()],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["robots_url", settings.prestashop_addon.robots_url],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["sitemap_lang", settings.prestashop_addon.sitemap_lang],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["flaresolverr_url", settings.flaresolverr.flaresolverr_url],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)",
        params!["user_agent", settings.flaresolverr.user_agent],
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