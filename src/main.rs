use std::sync::Arc;

use anyhow::{Context, Error};
use colored::*;
use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncReadExt, BufReader, BufWriter};

use wordpress::main::{Auth, CreateCategory, CreatePage, FindCategoryCustomPsAddonsCatId, FindPage};

use crate::utilities::{extract_data, extract_id_from_url};
use crate::wordpress::main::CreateProduct;

mod config;
mod extractors;

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

    // Load configuration settings
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    let flaresolverr_url = &config.flaresolverr.flaresolverr_url;
    let wordpress_url = &config.wordpress_api.wordpress_url;
    let username = &config.wordpress_api.username_api;
    let password = &config.wordpress_api.password_api;
    let status = &config.wordpress_page.status;
    let author = config.wordpress_page.author;
    let max_concurrency = config.base.max_concurrency;
    let wp = Arc::new(Auth::new(wordpress_url.to_string(), username.to_string(), password.to_string()));

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
        .delimiter(b'|')
        .quote(b'"')
        .double_quote(true)
        .create_writer(writer);

    // Write headers to the new CSV file
    csv_writer.write_record(headers).await?;

    // Process records from the CSV file
    let client = Client::new();

    let records_stream = csv_reader.records();
    records_stream
        .for_each_concurrent(max_concurrency, |record_result| {
            let wp = wp.clone();

            // Clone client for async block usage
            let client = client.clone();

            async move {
                match record_result {
                    Ok(record) => {
                        let url = record.get(0).unwrap_or_default();
                        let data = json!({
                            "cmd": "request.get",
                            "url": url,
                            "maxTimeout": 60000
                        });

                        // Send url csv at scrape to server flaresolverr
                        let response: Result<reqwest::Response, reqwest::Error> = client
                            .post(flaresolverr_url)
                            .header(reqwest::header::CONTENT_TYPE, "application/json")
                            .json(&data)
                            .send()
                            .await;

                        if let Ok(resp) = response {
                            if resp.status().is_success() {
                                let body: Result<config::config::FlareSolverrResponse, reqwest::Error> = resp.json().await;

                                if let Ok(body) = body {

                                    // Extract data scraped from server flaresolverr
                                    let extract_data = extract_data::extract_data(&body);

                                    let mut current_parent_id = config.wordpress_page.parent;

                                    // Create WooCommerce using hierarchy breadcrumbs extracted from scraped data
                                    let breadcrumbs = &extract_data.breadcrumbs;
                                    let last_index = breadcrumbs.len() - 1;

                                    for (index, breadcrumb) in breadcrumbs.iter().enumerate() {
                                        if let Some(id) = breadcrumb.get("id") {
                                            println!("Breadcrumb is: {}", id);

                                            if index == last_index {
                                                // Dernier breadcrumb est le produit
                                                println!("Creating product ...");

                                                let result = wp.create_product(
                                                    extract_data.title.to_string(),
                                                    "draft".to_string(),
                                                    "simple".to_string(),
                                                    true,
                                                    true,
                                                    extract_data.features.to_string(),
                                                    extract_data.description.to_string(),
                                                    extract_data.price_ht.to_string(),
                                                    vec![current_parent_id],
                                                    &extract_data.image_urls,
                                                    extract_data.product_id,
                                                    body.solution.url.to_string(),
                                                ).await;

                                                println!("Product creation response: {:?}", result);
                                                continue;
                                            }

                                            // todo: construire les categories et leurs enfants via le breadcrumb scrappé, vérifier si la catégorie existe, puis le dernier élément du breadcrumb sera le produit
                                            let id_ps_category = extract_id_from_url::extract_id_from_url(id);
                                            match wp.find_category_custom_ps_addons_cat_id(id_ps_category).await {
                                                Ok(category_info) => {
                                                    match category_info.status.as_ref() {
                                                        "found" => {
                                                            println!("Category found: {:?}", category_info.category_name);
                                                            if let Some(id) = category_info.category_id {
                                                                current_parent_id = id
                                                            }
                                                            continue;
                                                        }
                                                        "notfound" => {
                                                            println!("No category found with the given ID.");
                                                            let name = breadcrumb.get("name").unwrap().to_string();

                                                            match wp.create_category(name, current_parent_id, id_ps_category).await {
                                                                Ok(response) => {
                                                                    println!("Category created successfully: {}", response);

                                                                    if let Some(id_category) = response
                                                                        .get("id")
                                                                        .and_then(|v| v.as_i64())
                                                                    {
                                                                        current_parent_id = id_category as u32;
                                                                        println!(
                                                                            "Updating parent_id for next category: {}",
                                                                            current_parent_id
                                                                        );
                                                                    } else {
                                                                        eprintln!("{}", "Failed to extract parent_id from the response".red());
                                                                        continue;
                                                                    }
                                                                }
                                                                Err(e) => println!("Failed to create category: {}", e),
                                                            }
                                                        }
                                                        _ => println!("Error: {}", category_info.message),
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to find category: {}", e);
                                                    // Ajoute ici la logique pour gérer l'erreur ou terminer la tâche
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    eprintln!("Failed to deserialize response");
                                }
                            }
                        } else {
                            // todo: ajouter une logique pour gérer les erreurs de requête
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read CSV record: {}", e);
                    }
                }
            }
        }).await;

    Ok(())
}

async fn read_template_from_config() -> Result<String, std::io::Error> {
    // Load configuration settings
    //let config = match config::load_config() {
    //    Ok(cfg) => cfg,
    //    Err(e) => {
    //        eprintln!("Failed to load configuration: {}", e);
    //        return Err(e.into());
    //    }
    //};

    // Tentez d'ouvrir le fichier et gérez l'erreur éventuelle
    let mut file = AsyncFile::open("template_page.txt").await?;
    let mut template = String::new();
    // Lisez le fichier dans la chaîne 'template' et gérez l'erreur éventuelle
    file.read_to_string(&mut template).await?;
    Ok(template)
}
