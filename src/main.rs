use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use tokio;
use tokio::time::Instant;

use crate::config::configuration;
use crate::config::get_configuration::get_configuration_value_as_usize;
use crate::utilities::database;
use crate::utilities::extract_data;
use crate::utilities::sitemap;

mod config;
mod extractors;
mod utilities;
mod wordpress;
mod scrape_and_create_products;

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
    let db = match database::init::init().await {
        Ok(db) => {
            println!("{}", "Database initialized successfully".green());
            db
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to initialize database: {:?}", e).red());
            return Err(e.into());
        }
    };

    // Load configuration
    if let Err(e) = configuration::load_configuration(&db.conn, "Settings.toml").await {
        eprintln!("{}", format!("Failed to load configuration: {:?}", e).red());
        return Err(e.into());
    }

    // Update sitemap
    if let Err(e) = sitemap::sitemap_update::sitemap_update(&db.conn, 30).await {
        eprintln!("{}", format!("Failed to update sitemap: {:?}", e).red());
        return Err(e.into());
    }

    // Process URLs in batches
    let batch_size = get_configuration_value_as_usize(db.conn.clone(), "batch_size").await?;
    let max_concurrency = get_configuration_value_as_usize(db.conn.clone(), "max_concurrency").await?;
    // let wordpress_url = get_configuration_value(conn.clone(), "wordpress_url").await?;
    // let username_api = get_configuration_value(conn.clone(), "username_api").await?;
    // let password_api = get_configuration_value(conn.clone(), "password_api").await?;
    // let wp = Arc::new(Auth::new(wordpress_url, username_api, password_api));

    let start = Instant::now();
    if let Err(e) = scrape_and_create_products::process_urls_dynamically(db.conn.clone(), batch_size, max_concurrency).await {
        eprintln!("{}", format!("Failed to process URLs: {:?}", e).red());
        return Err(e.into());
    }

    let duration = start.elapsed();
    println!("{}", format!("Time taken to process URLs: {:?}", duration).green());

    Ok(())
}