use colored::*;
use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncReadExt, BufReader, BufWriter};

// WordPress functionality
use crate::wordpress::wp_create_page;
use crate::wordpress::wp_upload_image;

// Import modules
mod config;
mod scraping;
mod wordpress;
//mod add_page;
mod check_page;
mod extract_data;

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

                                            // Images process
                                            let mut formatted_strings = Vec::new();

                                            for image_url in &*extract_data.image_urls {
                                                println!("Uploading image from URL: {}", image_url);

                                                match wp_upload_image::upload_image_wordpress(
                                                    wordpress_url,
                                                    username,
                                                    password,
                                                    image_url,
                                                ).await {
                                                    Ok(response) => {
                                                        println!("Image uploaded successfully: {}", image_url);
                                                        if let Ok(media_response) = response.json::<MediaResponse>().await {
                                                            let formatted_string = format!("[fusion_image linktarget=\"_self\" image=\"{}\" image_id=\"{}|full\" /]", media_response.source_url, media_response.id);
                                                            formatted_strings.push(formatted_string);
                                                        } else {
                                                            eprintln!("Failed to parse response as JSON: {}", image_url);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to upload image: {}. Error: {}", image_url, e);
                                                        continue;
                                                    }
                                                }
                                            };

                                            let mut content = String::from(r#"[fusion_images order_by="desc" picture_size="auto" hover_type="none" autoplay="no" flex_align_items="center" columns="3" column_spacing="5" show_nav="yes" mouse_scroll="no" border="no" lightbox="yes" caption_style="off" caption_title_tag="2" caption_align_medium="none" caption_align_small="none" caption_align="none" hide_on_mobile="small-visibility,medium-visibility,large-visibility"]"#);
                                            let images_tags = formatted_strings.join("");
                                            content.push_str(&images_tags);
                                            content.push_str("\n[/fusion_images]");

                                            match read_template_from_config().await {
                                                Ok(template) => {
                                                    let template_modified = template
                                                        //.replace("[NAME]", name)
                                                        //.replace("[ID_PS_PRODUCT]", &*extract_data.product_id)
                                                        .replace("[PRICE_HT]", &*extract_data.price_ht)
                                                        .replace("[TITLE]", &*extract_data.title)
                                                        .replace("[DEV_NAME]", &*extract_data.developer_name)
                                                        .replace("[MODULE_VERSION]", &*extract_data.module_version)
                                                        .replace("[PUBLICATION_DATE]", &*extract_data.publication_date)
                                                        .replace("[LAST_UPDATE]", &*extract_data.last_update)
                                                        .replace("[PRESTASHOP_VERSION]", &*extract_data.ps_version_required)
                                                        .replace("[AS_OVERRIDES]", &*extract_data.with_override)
                                                        .replace("[IS_MULTISTORE]", &*extract_data.multistore_compatibility)
                                                        .replace("[DESCRIPTION]", &*extract_data.description)
                                                        .replace("[CARACTERISTIQUES]", &*extract_data.caracteristiques)
                                                        //.replace("#URL_MODULE", &body.solution.url)
                                                        .replace("[IMG_TAGS]", &images_tags);

                                                    template_rendered = template_modified;
                                                }
                                                Err(e) => eprintln!("Failed to read template: {}", e),
                                            };
                                            let template_final = template_rendered.as_str();

                                            match wp_create_page::create_wordpress_page(
                                                name, // Using breadcrumb name for page title
                                                template_final,
                                                &*extract_data.product_id,
                                                &body.solution.url,
                                                status,
                                                author,
                                                wordpress_url,
                                                username,
                                                password,
                                                current_parent_id,
                                            ).await
                                            {
                                                Ok(response) => {
                                                    // Processing the JSON response to extract the ID of the created page
                                                    match serde_json::from_str::<serde_json::Value>(
                                                        &response,
                                                    ) {
                                                        Ok(json_value) => {
                                                            println!("Page created successfully");
                                                            if let Some(id) = json_value
                                                                .get("id")
                                                                .and_then(|v| v.as_i64())
                                                            {
                                                                // Set this ID as the parent ID for the next pages
                                                                current_parent_id = id as i32;
                                                                println!(
                                                                    "Updating parent_id for the next pages: {}",
                                                                    current_parent_id
                                                                );
                                                            } else {
                                                                eprintln!("{}", "Failed to extract parent_id from the response".red());
                                                                continue;
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Error creating page 1: {}", e);
                                                            match serde_json::from_str::<serde_json::Value>(&e.to_string()) {
                                                                Ok(json) => {
                                                                    if let Some(message) = json["message"].as_str() {
                                                                        eprintln!("Detailed error: {}", message.red());
                                                                    }
                                                                }
                                                                Err(_) => eprintln!("{}", "Failed to parse error information as JSON".red())
                                                            }
                                                            continue;
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Error creating page 2: {}", e);
                                                    match serde_json::from_str::<serde_json::Value>(&e.to_string()) {
                                                        Ok(json) => {
                                                            if let Some(message) = json["message"].as_str() {
                                                                eprintln!("Detailed error: {}", message.red());
                                                            }
                                                        }
                                                        Err(_) => eprintln!("{}", "Failed to parse error information as JSON".red())
                                                    }
                                                    continue;
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
