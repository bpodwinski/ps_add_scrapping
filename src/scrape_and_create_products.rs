use std::env;
use std::error::Error;
use std::sync::{Arc, RwLockReadGuard};

use colored::Colorize;
use csv_async::AsyncReader;
use futures::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio::sync::Mutex;

use crate::config::config::AppConfig;
use crate::utilities::{extract_data, extract_id_from_url};
use crate::wordpress::main::{Auth, CreateCategory, CreateProduct, FindCategoryCustomPsAddonsCatId};

pub async fn scrape_and_create_products<'a>(
    config: Arc<AppConfig>,
    flaresolverr_url: &String,
    max_concurrency: Option<usize>,
    wp: Arc<Auth>,
    csv_reader: &mut AsyncReader<BufReader<File>>,
    client: Client,
) -> Result<(), Box<dyn Error>> {
    let records_stream = csv_reader.records().enumerate();
    records_stream.for_each_concurrent(max_concurrency, |(index, record_result)| {
        let wp = wp.clone();
        let client = client.clone();
        let config = config.clone();

        async move {
            match record_result {
                Ok(record) => {
                    //process_record(record, &wp, &client, env, &store, &config, flaresolverr_url).await.expect("TODO: panic message");
                }
                Err(e) => {
                    eprintln!("Failed to read CSV record: {}", e);
                }
            }
        }
    }).await;

    Ok(())
}

async fn process_record<'a>(
    record: csv_async::StringRecord,
    wp: &Arc<Auth>,
    client: &Client,
    config: &Arc<AppConfig>,
    flaresolverr_url: &String,
) -> Result<(), Box<dyn Error>> {
    let url = record.get(0).unwrap_or_default();
    let data = json!({
        "cmd": "request.get",
        "url": url,
        "maxTimeout": 60000
    });

    // Send URL to FlareSolverr for scraping
    let response = client.post(flaresolverr_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await;

    if let Ok(resp) = response {
        if resp.status().is_success() {
            let body: Result<crate::config::config::FlareSolverrResponse, reqwest::Error> = resp.json().await;
            if let Ok(body) = body {
                //process_body(body, wp, env, store, config).await?;
            } else {
                eprintln!("Failed to deserialize response");
            }
        }
    } else {
        eprintln!("Request failed: {:?}", response);
    }

    Ok(())
}

async fn process_body<'a>(
    body: crate::config::config::FlareSolverrResponse,
    wp: &Arc<Auth>,
    config: &Arc<AppConfig>,
) -> Result<(), Box<dyn Error>> {
    // Extract data from the scraped body
    let extract_data = extract_data::extract_data(&body);

    let mut current_parent_id = config.wordpress_page.parent;

    // Create WooCommerce products using breadcrumbs from scraped data
    let breadcrumbs = &extract_data.breadcrumbs;
    let last_breadcrumb_index = breadcrumbs.len() - 1;

    for (breadcrumb_index, breadcrumb) in breadcrumbs.iter().enumerate() {
        if let Some(id) = breadcrumb.get("id") {
            println!("Breadcrumb is: {}", id);

            if breadcrumb_index == last_breadcrumb_index {
                // Last breadcrumb is the product
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

                match result {
                    Ok(response) => {
                        println!("Product creation response: {:?}", response);

                        if let Some(http_status) = response["http_status"].as_u64() {
                            let product_id = extract_data.product_id as u64;

                            //store.put(&mut writer, &format!("product_{}_id", &product_id), &Value::U64(product_id)).expect("Failed to put data");
                            //store.put(&mut writer, &format!("product_{}_http_code", &product_id), &Value::U64(http_status)).expect("Failed to put data");
                            //store.put(&mut writer, &format!("product_{}_date_modified", &product_id), &Value::Str(response["body"]["date_modified"].as_str().unwrap_or(""))).expect("Failed to put data");
                            //store.put(&mut writer, &format!("product_{}_ps_url", &product_id), &Value::Str(body.solution.url.as_str())).expect("Failed to put data");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }

            // Handle category creation and updating
            let id_ps_category = extract_id_from_url::extract_id_from_url(id);
            match wp.find_category_custom_ps_addons_cat_id(id_ps_category).await {
                Ok(category_info) => {
                    match category_info.status.as_ref() {
                        "found" => {
                            println!("Category found: {:?}", category_info.category_name);
                            if let Some(id) = category_info.category_id {
                                current_parent_id = id;
                            }
                        }
                        "notfound" => {
                            println!("No category found with the given ID.");
                            let name = breadcrumb.get("name").unwrap().to_string();

                            match wp.create_category(name, current_parent_id, id_ps_category).await {
                                Ok(response) => {
                                    println!("Category created successfully: {}", response);

                                    if let Some(id_category) = response.get("id").and_then(|v| v.as_i64()) {
                                        current_parent_id = id_category as u32;
                                        println!("Updating parent_id for next category: {}", current_parent_id);
                                    } else {
                                        eprintln!("{}", "Failed to extract parent_id from the response".red());
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
                }
            }
        }
    }

    Ok(())
}
