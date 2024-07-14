use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use tokio;
use tokio::time::Instant;

use crate::config::configuration;
use crate::config::get_configuration::{get_configuration_value_as_i64, get_configuration_value_as_usize};
use crate::utilities::database;
use crate::utilities::extract_data;
use crate::utilities::sitemap;

mod config;
mod extractors;
mod utilities;
mod wordpress;
mod process;

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
            eprintln!("{}", format!("Failed to initialize database: {:?}", e).red());
            return Err(e.into());
        }
    };
    let db = &db_init.conn;

    // Load configuration
    if let Err(e) = configuration::load_configuration(&db, "Settings.toml").await {
        eprintln!("{}", format!("Failed to load configuration: {:?}", e).red());
        return Err(e.into());
    }

    // Update sitemap
    let sitemap_frequency_update = get_configuration_value_as_i64(&db, "sitemap_frequency_update").await?;

    if let Err(e) = sitemap::sitemap_update::sitemap_update(&db, sitemap_frequency_update).await {
        eprintln!("{}", format!("Failed to update sitemap: {:?}", e).red());
        return Err(e.into());
    }

    // Process URLs in batches
    let batch_size = get_configuration_value_as_usize(&db, "batch_size").await?;
    let max_concurrency = get_configuration_value_as_usize(&db, "max_concurrency").await?;

    let start = Instant::now();
    if let Err(e) = process::process_urls_dynamically(&db, batch_size, max_concurrency).await {
        eprintln!("{}", format!("Failed to process URLs: {:?}", e).red());
        return Err(e.into());
    }

    let duration = start.elapsed();
    println!("{}", format!("Time taken to process URLs: {:?}", duration).green());

    Ok(())
}