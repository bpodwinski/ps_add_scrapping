use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use futures::{stream, StreamExt};
use reqwest::Client;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;
use tokio::sync::Mutex;
use tokio::task;

use crate::config::get_configuration::{get_configuration_value, get_configuration_value_as_i64};
use crate::utilities::{extract_data, extract_id_from_url};
use crate::utilities::generate_random_delay::generate_random_delay;
use crate::wordpress::main::{Auth, CreateCategory, CreateProduct, FindCategoryCustomPsAddonsCatId};

/// Processes URLs in batches, executing a fixed number of tasks concurrently.
///
/// # Arguments
///
/// * `conn` - A shared, locked database connection.
/// * `batch_size` - The number of URLs to fetch in each batch.
/// * `max_concurrent_tasks` - The maximum number of concurrent tasks.
///
/// # Returns
///
/// `Ok(())` if all URLs are processed successfully, or an error if any task fails.
pub async fn process_urls_dynamically(
    conn: Arc<Mutex<Connection>>,
    batch_size: usize,
    max_concurrent_tasks: usize,
) -> Result<()> {
    let mut offset = 0;

    loop {
        let urls = get_urls_batch(conn.clone(), offset, batch_size).await?;

        if urls.is_empty() {
            break;
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

/// Fetches a batch of URLs from the database.
///
/// # Arguments
///
/// * `conn` - A shared, locked database connection.
/// * `offset` - The starting point to fetch URLs from.
/// * `limit` - The maximum number of URLs to fetch.
///
/// # Returns
///
/// A vector of URLs if successful, or an error if the operation fails.
async fn get_urls_batch(
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

/// Processes a given URL by checking the last modification date and scraping it if necessary.
///
/// # Arguments
///
/// * `conn` - A shared, locked database connection.
/// * `url` - The URL to be processed.
///
/// # Returns
///
/// An empty `Result` if successful, or an error if the operation fails.
async fn process_url(
    conn: Arc<Mutex<Connection>>,
    url: String,
) -> Result<()> {

    // Checks when URL was last scraped, then doesn't process it if it was scraped recently
    // If URL was scraped with an HTTP error (e.g., 403, 500), process URL
    let age_url = get_configuration_value_as_i64(conn.clone(), "age_url").await?;

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
                    println!("{}", format!("Skipping URL as it was modified less than {} hours ago: {}", age_url, url).cyan());
                    return Ok(());
                }
            }
        }
    }

    // Send URL to scraping via FlareSolverr
    let (status, body) = send_url_to_flaresolverr(conn.clone(), &url).await?;

    // FlareSolverr scraping failed
    if !status.is_success() {
        eprintln!("{}", format!("Failed to create product with status {}: {:?}", status, body).red());

        // Update database
        let http_code = status.as_u16();
        let date_modified = Utc::now().to_rfc3339();
        update_url_in_database(conn.clone(), &url, &date_modified, http_code).await?;

        // Generate random delay
        generate_random_delay(500, 6000).await;

        return Err(anyhow::anyhow!("Failed to process URL due to FlareSolverr error"));
    }

    // FlareSolverr scraping success
    println!("{}", "Scraping success!".green());

    let extract_data = extract_data::extract_data(&body);

    // Create WooCommerce products using breadcrumbs from scraped data
    let wordpress_url = get_configuration_value(conn.clone(), "wordpress_url").await?;
    let username_api = get_configuration_value(conn.clone(), "username_api").await?;
    let password_api = get_configuration_value(conn.clone(), "password_api").await?;

    let wp = Arc::new(Auth::new(wordpress_url, username_api, password_api));
    let breadcrumbs = &extract_data.breadcrumbs;
    let last_breadcrumb_index = breadcrumbs.len() - 1;

    // Process breadcrumb for create category and product
    let mut current_wordpress_parent = get_configuration_value_as_i64(conn.clone(), "wordpress_parent").await?;

    for (breadcrumb_index, breadcrumb) in breadcrumbs.iter().enumerate() {
        if let Some(id) = breadcrumb.get("id") {

            // Create or update category
            let id_ps_category = extract_id_from_url::extract_id_from_url(id);

            match wp.find_category_custom_ps_addons_cat_id(id_ps_category).await {
                Ok(category_info) => {
                    match category_info.status.as_ref() {
                        "found" => {
                            println!("{}", format!("Category found: {:?}", category_info.category_name.as_deref().unwrap_or("Unknown")).cyan());

                            if let Some(id) = category_info.category_id {
                                current_wordpress_parent = id as i64;
                            }
                        }
                        "notfound" => {
                            println!("{}", "No category found with the given ID".cyan());
                            let name = breadcrumb.get("name").unwrap().to_string();

                            match wp.create_category(name, current_wordpress_parent as u32, id_ps_category).await {
                                Ok(response) => {
                                    println!("{}", format!("Category created successfully: {:?}", category_info.category_name.as_deref().unwrap_or("Unknown")).green());

                                    if let Some(id_category) = response.get("id").and_then(|v| v.as_i64()) {
                                        current_wordpress_parent = id_category;
                                    } else {
                                        eprintln!("{}", "Failed to extract parent_id from the response".red());
                                    }
                                }
                                Err(e) => eprintln!("{}", format!("Failed to create category: {:?}", e).red()),
                            }
                        }
                        _ => eprintln!("{}", format!("Failed to create category: {:?}", category_info.message).red()),
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Failed to find category: {:?}", e).red());
                }
            }

            // Create product at last breadcrumb
            if breadcrumb_index == last_breadcrumb_index {
                println!("{}", "Creating product ...".cyan());

                let create_product = match wp.create_product(
                    extract_data.title.to_string(),
                    "draft".to_string(),
                    "simple".to_string(),
                    true,
                    true,
                    extract_data.features.to_string(),
                    extract_data.description.to_string(),
                    extract_data.price_ht.to_string(),
                    vec![current_wordpress_parent as u32],
                    &extract_data.image_urls,
                    extract_data.product_id,
                    body.solution.url.to_string(),
                ).await {
                    Ok(response) => {
                        println!("{}", format!("Product created successfully: {:?}", response).green());

                        if let Some(http_status) = response["http_status"].as_u64() {
                            let product_id = extract_data.product_id as u64;

                            //store.put(&mut writer, &format!("product_{}_id", &product_id), &Value::U64(product_id)).expect("Failed to put data");
                            //store.put(&mut writer, &format!("product_{}_http_code", &product_id), &Value::U64(http_status)).expect("Failed to put data");
                            //store.put(&mut writer, &format!("product_{}_date_modified", &product_id), &Value::Str(response["body"]["date_modified"].as_str().unwrap_or(""))).expect("Failed to put data");
                            //store.put(&mut writer, &format!("product_{}_ps_url", &product_id), &Value::Str(body.solution.url.as_str())).expect("Failed to put data");
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Failed to create product: {:?}", e).red());
                    }
                };
            }
        }
    }

    // Generate random delay
    generate_random_delay(1000, 8000).await;

    // Update database
    let date_modified = Utc::now().to_rfc3339();
    let http_code = status.as_u16();
    update_url_in_database(conn.clone(), &url, &date_modified, http_code).await?;

    Ok(())
}

/// Sends a URL to FlareSolverr for scraping.
///
/// # Arguments
///
/// * `conn` - A reference to the database connection.
/// * `url` - The URL to be scraped.
///
/// # Returns
///
/// If successful, returns the status and body of the response.
async fn send_url_to_flaresolverr(
    conn: Arc<Mutex<Connection>>,
    url: &str,
) -> Result<(
    reqwest::StatusCode,
    extract_data::FlareSolverrResponse
)> {
    // Create an HTTP client
    let client = Client::new();

    // Prepare the JSON payload
    let data = json!({
        "cmd": "request.get",
        "url": url,
        "maxTimeout": 60000
    });

    // Get the FlareSolverr URL from the configuration
    let flaresolverr_url = get_configuration_value(conn.clone(), "flaresolverr_url").await?;

    // Send the request to FlareSolverr
    let response = client
        .post(&flaresolverr_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&data)
        .send()
        .await
        .context("Failed to send request to Flaresolverr")?;

    // Get the status and body of the response
    let status = response.status();
    let body: extract_data::FlareSolverrResponse = response.json().await.context("Failed to read response body")?;

    Ok((status, body))
}

/// Updates the `date_modified` and `http_code` fields for a given URL in the database.
///
/// # Arguments
///
/// * `conn` - A shared, locked database connection.
/// * `url` - The URL to update.
/// * `date_modified` - The new modification date in RFC3339 format.
/// * `http_code` - The new HTTP status code.
///
/// # Returns
///
/// `Ok(())` if the update is successful, or an error if it fails.
async fn update_url_in_database(
    conn: Arc<Mutex<Connection>>,
    url: &str,
    date_modified: &str,
    http_code: u16,
) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        "UPDATE urls SET date_modified = ?1, http_code = ?2 WHERE url = ?3",
        params![date_modified, http_code, url],
    )?;
    Ok(())
}