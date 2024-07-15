use std::env;

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use tokio;
use tokio::time::Instant;

use crate::config::configuration;
use crate::config::get_configuration::{
    get_configuration_value_as_i64, get_configuration_value_as_usize,
};
use crate::utilities::database;
use crate::utilities::sitemap;

mod config;
mod extractors;
mod process;
mod utilities;
mod wordpress;

#[derive(Deserialize, Serialize, Debug)]
struct JsonResponse {
    status: String,
    message: String,
    page_id: Option<i32>,
    title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct MediaResponse {
    id: u64,
    guid: RenderedItem,
    source_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RenderedItem {
    rendered: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Initialize SQLite
    let db_init = match database::init::init().await {
        Ok(db) => {
            println!("{}", "Database initialized successfully".green());
            db
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!("Failed to initialize database: {:?}", e).red()
            );
            return Err(e.into());
        }
    };
    let db = &db_init.conn;

    // Check if Settings.toml exists in the current working directory
    let mut current_dir = env::current_exe().context("Failed to get current executable path")?;
    current_dir.pop();
    let config_path = current_dir.join("Settings.toml");
    if !config_path.exists() {
        eprintln!("{}", "Settings.toml file not found".red());
        return Err(Box::from(anyhow::anyhow!("Settings.toml file not found")));
    }

    // Load configuration
    if let Err(e) = configuration::load_configuration(&db, config_path.to_str().unwrap()).await {
        eprintln!("{}", format!("Failed to load configuration: {:?}", e).red());
        return Err(e.into());
    }

    // Update sitemap
    let sitemap_frequency_update =
        get_configuration_value_as_i64(&db, "sitemap_frequency_update").await?;

    if let Err(e) = sitemap::sitemap_update::sitemap_update(&db, sitemap_frequency_update).await {
        eprintln!("{}", format!("Failed to update sitemap: {:?}", e).red());
        return Err(e.into());
    }

    // Process URLs
    let batch_size = get_configuration_value_as_usize(&db, "batch_size").await?;
    let max_concurrency = get_configuration_value_as_usize(&db, "max_concurrency").await?;

    let start = Instant::now();

    if let Err(e) = process::process_urls_dynamically(&db, batch_size, max_concurrency).await {
        eprintln!("{}", format!("Failed to process URLs: {:?}", e).red());
        return Err(e.into());
    }

    let duration = start.elapsed();
    println!(
        "{}",
        format!("Time to process URLs: {:?}", duration).green()
    );

    Ok(())
}
