use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};

// Import modules
mod config;
mod scraping;
mod wordpress;

// Scraping utilities
use scraping::{
    extract_breadcrumb,
    extract_price_ht,
    extract_product_id,
    extract_title,
};

// WordPress functionality
use wordpress::{
    create_wordpress_page,
    check_page_exists,
};

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
            let client = client.clone(); // Clone client for async block usage
            
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
                                    let ps_url = &body.solution.url;
                                    let title = extract_title::extract_title(&body.solution.response);
                                    let product_id =
                                        extract_product_id::extract_product_id(&body.solution.response);
                                    let price_ht =
                                        extract_price_ht::extract_price_ht(&body.solution.response);
                                    let breadcrumbs =
                                        extract_breadcrumb::extract_breadcrumb(&body.solution.response);

                                    match product_id {
                                        Some(value) => println!("Product ID: {}", value),
                                        None => println!("Product ID: {}", ""),
                                    }

                                    match price_ht {
                                        Some(price) => println!("Price HT: {}", price),
                                        None => println!("Price HT: {}", ""),
                                    }

                                    println!("Title: {}", title);
                                    println!("PS Url: {}", ps_url);

                                    // Builds template for WordPress pages using data scraped
                                    let content = "<p>This is a test page</p>";
                                    let status = "draft";
                                    let mut current_parent_id = config.base.root_id_page;

                                    // Create WordPress pages using hierarchy breadcrumbs extracted from scraped data
                                    for breadcrumb in &breadcrumbs {
                                        if let Some(name) = breadcrumb.get("name") {

                                            println!("Breadcrumb name: {}", name);

                                            // Check if page already exist
                                            let check_response = check_page_exists::check_page_exists(
                                                name,
                                                wordpress_url,
                                                username,
                                                password,
                                            ).await;

                                            match check_response {
                                                Ok(Some(response)) => {
                                                    println!("Page already exists, response: {}", response);
                                                    continue;
                                                },
                                                Ok(None) => {
                                                    println!("No existing page found, proceeding to create a new page");
                                                },
                                                Err(e) => {
                                                    eprintln!("Error checking if page exists: {}", e);
                                                    continue;
                                                }
                                            }

                                            // Create WordPress page if not exists
                                            match create_wordpress_page::create_wordpress_page(
                                                name, // Using breadcrumb name for page title
                                                content,
                                                status,
                                                wordpress_url,
                                                username,
                                                password,
                                                current_parent_id,
                                            )
                                            .await
                                            {
                                                Ok(response) => {
                                                    println!("Page created successfully");
                                                    //println!("Page created successfully, raw response: {}", response);

                                                    // Processing the JSON response to extract the ID of the created page
                                                    match serde_json::from_str::<serde_json::Value>(
                                                        &response,
                                                    ) {
                                                        Ok(json_value) => {
                                                            if let Some(id) = json_value
                                                                .get("id")
                                                                .and_then(|v| v.as_i64())
                                                            {
                                                                current_parent_id = id as i32; // Set this ID as the parent ID for the next pages
                                                                println!(
                                                                    "Updating parent_id for the next pages: {}",
                                                                    current_parent_id
                                                                );
                                                            } else {
                                                                eprintln!("Failed to extract parent_id from the response");
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Error while parsing the JSON response: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(e) => eprintln!("Error creating page: {:?}", e),
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
