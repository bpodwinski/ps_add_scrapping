use std::collections::HashMap;

use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncReadExt, BufReader, BufWriter};

// Scraping utilities
use scraping::{
    extract_breadcrumb,
    extract_price_ht,
    extract_product_id,
    extract_title,
};
// WordPress functionality
use wordpress::{
    check_page_exists,
    create_wordpress_page,
};

use crate::config::config::FlareSolverrResponse;

// Import modules
mod config;
mod scraping;
mod wordpress;
mod add_page;
mod check_page;
mod extract_data;

#[derive(Deserialize, Serialize, Debug)]
struct JsonResponse {
    status: String,
    message: String,
    page_id: Option<i32>,
    title: Option<String>,
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
    let max_concurrency = 1;

    let records_stream = csv_reader.records();
    records_stream
        .for_each_concurrent(max_concurrency, |record_result| {

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

                                //let body: FlareSolverrResponse = resp.json().await;
                                let body: Result<config::config::FlareSolverrResponse, reqwest::Error> = resp.json().await;

                                if let Ok(body) = body {

                                    // Extract data scraped from server flaresolverr
                                    let extract_data = extract_data::extract_data(&body);

                                    let mut current_parent_id = config.wordpress_page.parent;

                                    // Create WordPress pages using hierarchy breadcrumbs extracted from scraped data
                                    for breadcrumb in extract_data.breadcrumbs {
                                        if let Some(name) = breadcrumb.get("name") {
                                            println!("Breadcrumb name: {}", name);

                                            if check_page::check_page(
                                                wordpress_url,
                                                username,
                                                password,
                                                name,
                                            ).await { continue; }

                                            // Builds template for WordPress pages using data scraped
                                            let mut template_rendered = String::new();

                                            match read_template_from_config().await {
                                                Ok(mut template) => {
                                                    let template_modified = template
                                                        .replace("[NAME]", name)
                                                        .replace("[ID_PS_PRODUCT]", &*extract_data.product_id)
                                                        .replace("[PRICE_HT]", &*extract_data.price_ht)
                                                        .replace("[TITLE]", &*extract_data.title)
                                                        .replace("[DEV_NAME]", &*extract_data.developer_name)
                                                        .replace("[MODULE_VERSION]", "module_version")
                                                        .replace("[PUBLICATION_DATE]", "pub_date")
                                                        .replace("[LAST_UPDATE]", "last_update")
                                                        .replace("[PRESTASHOP_VERSION]", "ps_version")
                                                        .replace("[AS_OVERRIDES]", "as_overrides")
                                                        .replace("[IS_MULTISTORE]", "is_multistores")
                                                        .replace("[DESCRIPTION]", "desc")
                                                        .replace("[CARACTERISTIQUES]", "caract")
                                                        .replace("#URL_MODULE", "url_module")
                                                        .replace("[IMG_TAGS]", "img");
                                                    template_rendered = template_modified;
                                                }
                                                Err(e) => eprintln!("Failed to read template: {}", e),
                                            };

                                            let template_final = template_rendered.as_str();

                                            add_page::add_page(
                                                wordpress_url,
                                                username,
                                                password,
                                                template_final,
                                                &*extract_data.product_id,
                                                status,
                                                config.wordpress_page.author,
                                                current_parent_id,
                                                name,
                                            ).await;
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
