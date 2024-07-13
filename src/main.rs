use anyhow::{Context, Result};
use colored::*;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncReadExt;

use wordpress::main::{CreateCategory, CreatePage, FindCategoryCustomPsAddonsCatId, FindPage};

use crate::config::configuration;
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
    // let config = match config::load_config() {
    //     Ok(cfg) => cfg,
    //     Err(e) => {
    //         eprintln!("Failed to load configuration: {}", e);
    //         return Err(e.into());
    //     }
    // };
    // let config = Arc::new(config);
    // let flaresolverr_url = &config.flaresolverr.flaresolverr_url;
    // let wordpress_url = &config.wordpress_api.wordpress_url;
    // let username = &config.wordpress_api.username_api;
    // let password = &config.wordpress_api.password_api;
    // let status = &config.wordpress_page.status;
    // let author = config.wordpress_page.author;
    // let max_concurrency = config.base.max_concurrency;
    // let wp = Arc::new(Auth::new(wordpress_url.to_string(), username.to_string(), password.to_string()));

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

    //scrape_and_create_products::scrape_and_create_products(config.clone(), &flaresolverr_url.to_string(), max_concurrency, wp, &mut csv_reader, client).await.expect("TODO: panic message");
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
