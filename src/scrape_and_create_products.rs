use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use futures::{stream, StreamExt};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use reqwest::Client;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{Duration, sleep};

use crate::config::get_configuration::{get_configuration_value, get_configuration_value_as_i64};
use crate::utilities::generate_random_delay::generate_random_delay;

pub async fn get_urls_batch(
    conn: Arc<Mutex<Connection>>,
    offset: usize,
    limit: usize,
) -> Result<Vec<String>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT url FROM urls ORDER BY id LIMIT ?1 OFFSET ?2")?;
    let rows = stmt.query_map(params![limit, offset], |row| row.get(0))?;

    let mut urls = Vec::new();
    for url in rows {
        urls.push(url?);
    }

    Ok(urls)
}

pub async fn process_urls_dynamically(
    conn: Arc<Mutex<Connection>>,
    batch_size: usize,
    max_concurrent_tasks: usize,
) -> Result<()> {
    let mut offset = 0;

    loop {
        let urls = get_urls_batch(conn.clone(), offset, batch_size).await?;

        if urls.is_empty() {
            break; // Plus d'URLs à traiter
        }

        // Create a stream to process the URLs in parallel
        let tasks = stream::iter(urls.into_iter().map(|url| {
            let conn = conn.clone();
            task::spawn(async move {
                if let Err(e) = process_url(conn, url).await {
                    eprintln!("Failed to process URL: {:?}", e);
                }
            })
        }));

        // Buffer the tasks to ensure a fixed number of tasks are running at any time
        tasks.buffer_unordered(max_concurrent_tasks).for_each(|_| async {}).await;

        offset += batch_size;
    }

    Ok(())
}

async fn process_url(
    conn: Arc<Mutex<Connection>>,
    url: String,
) -> Result<()> {
    let age_url = get_configuration_value_as_i64(conn.clone(), "age_url").await?;

    // Checks when URL was last scraped, then doesn't process it if it was scraped recently
    // If URL was scraped with an HTTP error (e.g., 403, 500), process URL
    {
        let conn = conn.lock().await;
        let mut stmt = conn.prepare("SELECT date_modified, http_code FROM urls WHERE url = ?1")?;
        let row = stmt.query_row([url.as_str()], |row| {
            let date_modified: Option<String> = row.get(0)?;
            let http_code: Option<i32> = row.get(1)?;
            Ok((date_modified, http_code))
        }).optional()?;

        if let Some((date_modified, http_code)) = row {

            // Skip URL if it was modified less than `age_url` hours ago and http_code is 200
            if let Some(date_modified) = date_modified {
                let last_mod_date = DateTime::parse_from_rfc3339(&date_modified)?;
                let now = Utc::now();
                let last_mod_date_utc = last_mod_date.with_timezone(&Utc);
                let hours_difference = (now - last_mod_date_utc).num_hours();

                if hours_difference < age_url && http_code == Some(200) {
                    println!("Skipping URL as it was modified less than {} hours ago: {}", age_url, url);
                    return Ok(());
                }
            }
        }
    }

    let client = Client::new();

    let data = json!({
            "cmd": "request.get",
            "url": url,
            "maxTimeout": 60000
        });

    // Send URL to scraping via FlareSolverr
    let flaresolverr_url = get_configuration_value(conn.clone(), "flaresolverr_url").await?;

    let response = client
        .post(&flaresolverr_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await
        .context("Failed to send request to Flaresolverr")?;

    let status = response.status();
    let body = response.text().await.context("Failed to read response body")?;

    // FlareSolverr scraping failed
    if !status.is_success() {
        eprintln!("Error: Status: {}, Body: {}", status, body);

        // Update database
        let http_code = status.as_u16();
        let date_modified = Utc::now().to_rfc3339();

        let conn = conn.lock().await;
        conn.execute(
            "UPDATE urls SET date_modified = ?1, http_code = ?2 WHERE url = ?3",
            rusqlite::params![date_modified, http_code, url],
        )?;

        // Generate random delay
        generate_random_delay(500, 6000).await;

        return Err(anyhow::anyhow!("Failed to process URL due to FlareSolverr error"));
    }

    println!("Status: {}, Body: {}", status, body);

    // Generate random delay
    generate_random_delay(1000, 8000).await;

    // Update database
    let date_modified = Utc::now().to_rfc3339();
    let http_code = status.as_u16();

    let conn = conn.lock().await;
    conn.execute(
        "UPDATE urls SET date_modified = ?1, http_code = ?2 WHERE url = ?3",
        rusqlite::params![date_modified, http_code, url],
    )?;

    Ok(())
}

// pub async fn scrape_and_create_products<'a>(
//     conn: Arc<Mutex<Connection>>,
//     max_concurrency: Option<usize>,
//     wp: Arc<Auth>,
//     csv_reader: &mut AsyncReader<BufReader<File>>,
//     client: Client,
// ) -> Result<(), Box<dyn Error>> {
//     let flaresolverr_url = get_configuration_value(conn.clone(), "flaresolverr_url").await?;
//     let records_stream = csv_reader.records().enumerate();
//
//     records_stream.for_each_concurrent(max_concurrency, |(index, record_result)| {
//         let wp = wp.clone();
//         let client = client.clone();
//         let config = config.clone();
//
//         async move {
//             match record_result {
//                 Ok(record) => {
//                     process_record(record, &wp, &client, env, &store, &config, flaresolverr_url).await.expect("TODO: panic message");
//                 }
//                 Err(e) => {
//                     eprintln!("Failed to read CSV record: {}", e);
//                 }
//             }
//         }
//     }).await;
//
//     Ok(())
// }

// async fn process_record<'a>(
//     record: csv_async::StringRecord,
//     wp: &Arc<Auth>,
//     client: &Client,
//     config: &Arc<AppConfig>,
//     flaresolverr_url: &String,
// ) -> Result<(), Box<dyn Error>> {
//     let url = record.get(0).unwrap_or_default();
//     let data = json!({
//         "cmd": "request.get",
//         "url": url,
//         "maxTimeout": 60000
//     });
//
//     // Send URL to FlareSolverr for scraping
//     let response = client.post(flaresolverr_url)
//         .header(reqwest::header::CONTENT_TYPE, "application/json")
//         .json(&data)
//         .send()
//         .await;
//
//     if let Ok(resp) = response {
//         if resp.status().is_success() {
//             let body: Result<crate::config::config::FlareSolverrResponse, reqwest::Error> = resp.json().await;
//             if let Ok(body) = body {
//                 //process_body(body, wp, env, store, config).await?;
//             } else {
//                 eprintln!("Failed to deserialize response");
//             }
//         }
//     } else {
//         eprintln!("Request failed: {:?}", response);
//     }
//
//     Ok(())
// }

// async fn process_body<'a>(
//     body: crate::config::config::FlareSolverrResponse,
//     wp: &Arc<Auth>,
//     config: &Arc<AppConfig>,
// ) -> Result<(), Box<dyn Error>> {
//     // Extract data from the scraped body
//     let extract_data = extract_data::extract_data(&body);
//
//     let mut current_parent_id = config.wordpress_page.parent;
//
//     // Create WooCommerce products using breadcrumbs from scraped data
//     let breadcrumbs = &extract_data.breadcrumbs;
//     let last_breadcrumb_index = breadcrumbs.len() - 1;
//
//     for (breadcrumb_index, breadcrumb) in breadcrumbs.iter().enumerate() {
//         if let Some(id) = breadcrumb.get("id") {
//             println!("Breadcrumb is: {}", id);
//
//             if breadcrumb_index == last_breadcrumb_index {
//                 // Last breadcrumb is the product
//                 println!("Creating product ...");
//
//                 let result = wp.create_product(
//                     extract_data.title.to_string(),
//                     "draft".to_string(),
//                     "simple".to_string(),
//                     true,
//                     true,
//                     extract_data.features.to_string(),
//                     extract_data.description.to_string(),
//                     extract_data.price_ht.to_string(),
//                     vec![current_parent_id],
//                     &extract_data.image_urls,
//                     extract_data.product_id,
//                     body.solution.url.to_string(),
//                 ).await;
//
//                 match result {
//                     Ok(response) => {
//                         println!("Product creation response: {:?}", response);
//
//                         if let Some(http_status) = response["http_status"].as_u64() {
//                             let product_id = extract_data.product_id as u64;
//
//                             //store.put(&mut writer, &format!("product_{}_id", &product_id), &Value::U64(product_id)).expect("Failed to put data");
//                             //store.put(&mut writer, &format!("product_{}_http_code", &product_id), &Value::U64(http_status)).expect("Failed to put data");
//                             //store.put(&mut writer, &format!("product_{}_date_modified", &product_id), &Value::Str(response["body"]["date_modified"].as_str().unwrap_or(""))).expect("Failed to put data");
//                             //store.put(&mut writer, &format!("product_{}_ps_url", &product_id), &Value::Str(body.solution.url.as_str())).expect("Failed to put data");
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error: {}", e);
//                     }
//                 }
//             }
//
//             // Handle category creation and updating
//             let id_ps_category = extract_id_from_url::extract_id_from_url(id);
//             match wp.find_category_custom_ps_addons_cat_id(id_ps_category).await {
//                 Ok(category_info) => {
//                     match category_info.status.as_ref() {
//                         "found" => {
//                             println!("Category found: {:?}", category_info.category_name);
//                             if let Some(id) = category_info.category_id {
//                                 current_parent_id = id;
//                             }
//                         }
//                         "notfound" => {
//                             println!("No category found with the given ID.");
//                             let name = breadcrumb.get("name").unwrap().to_string();
//
//                             match wp.create_category(name, current_parent_id, id_ps_category).await {
//                                 Ok(response) => {
//                                     println!("Category created successfully: {}", response);
//
//                                     if let Some(id_category) = response.get("id").and_then(|v| v.as_i64()) {
//                                         current_parent_id = id_category as u32;
//                                         println!("Updating parent_id for next category: {}", current_parent_id);
//                                     } else {
//                                         eprintln!("{}", "Failed to extract parent_id from the response".red());
//                                     }
//                                 }
//                                 Err(e) => println!("Failed to create category: {}", e),
//                             }
//                         }
//                         _ => println!("Error: {}", category_info.message),
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("Failed to find category: {}", e);
//                 }
//             }
//         }
//     }
//
//     Ok(())
// }
