use std::sync::Arc;

use anyhow::{Context, Result};
use colored::*;
use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncReadExt, BufReader, BufWriter};

use wordpress::main::{Auth, CreateCategory, CreatePage, FindCategoryCustomPsAddonsCatId, FindPage};

use crate::utilities::database;
use crate::utilities::extract_data;
use crate::utilities::sitemap;
use crate::wordpress::main::CreateProduct;

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

    // Load configuration
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };
    let config = Arc::new(config);
    let flaresolverr_url = &config.flaresolverr.flaresolverr_url;
    let wordpress_url = &config.wordpress_api.wordpress_url;
    let username = &config.wordpress_api.username_api;
    let password = &config.wordpress_api.password_api;
    let status = &config.wordpress_page.status;
    let author = config.wordpress_page.author;
    let max_concurrency = config.base.max_concurrency;
    let wp = Arc::new(Auth::new(wordpress_url.to_string(), username.to_string(), password.to_string()));

    // Initialize SQLite
    let conn = match database::init_sqlite::init_sqlite() {
        Ok(conn) => {
            println!("{}", "Database initialized successfully".green());
            conn
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to initialize database: {:?}", e).red());
            return Err(e.into());
        }
    };

    // Sitemap update
    sitemap::sitemap_update::sitemap_update(&conn, 7).await?;

    // Setup CSV file reading
    let file = AsyncFile::open(&config.file.source_data).await?;
    let reader = BufReader::new(file);
    let mut csv_reader = AsyncReaderBuilder::new().create_reader(reader);

    // Read headers from CSV file
    let headers = csv_reader.headers().await?;

    // Setup CSV file writing
    let file_out = AsyncFile::create(&config.file.processing_data).await?;
    let writer = BufWriter::new(file_out);

    let mut csv_writer = AsyncWriterBuilder::new()
        .delimiter(b';')
        .quote(b'"')
        .double_quote(true)
        .create_writer(writer);

    // Write headers to the new CSV file
    csv_writer.write_record(headers).await?;

    // Process records from the CSV file
    let client = Client::new();

    scrape_and_create_products::scrape_and_create_products(config.clone(), &flaresolverr_url.to_string(), max_concurrency, wp, &mut csv_reader, client).await.expect("TODO: panic message");
    Ok(())
}

async fn read_template_from_config() -> Result<String, std::io::Error> {
    // Tentez d'ouvrir le fichier et gérez l'erreur éventuelle
    let mut file = AsyncFile::open("template_page.txt").await?;
    let mut template = String::new();
    // Lisez le fichier dans la chaîne 'template' et gérez l'erreur éventuelle
    file.read_to_string(&mut template).await?;
    Ok(template)
}
