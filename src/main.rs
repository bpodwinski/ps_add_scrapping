use config::{Config, ConfigError, Environment, File};
use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use serde_json::json;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{BufReader, BufWriter};
use tokio::time::{self, Duration};

// Utils import
mod utils;
use utils::create_wordpress_page;
use utils::extract_breadcrumb;
//use utils::extract_caracteristiques;
//use utils::extract_description;
//use utils::extract_developer_name;
//use utils::extract_last_update;
//use utils::extract_meta_description;
//use utils::extract_module_version;
//use utils::extract_multistore_compatibility;
//use utils::extract_override;
use utils::extract_price_ht;
use utils::extract_product_id;
//use utils::extract_publication_date;
use utils::extract_title;

#[derive(Deserialize)]
struct FlareSolverrResponse {
    solution: Solution,
}

#[derive(Deserialize)]
struct Solution {
    url: String,
    response: String,
}

#[derive(Deserialize)]
struct AppConfig {
    base: BaseConfig,
    file: FileConfig,
    flaresolverr: FlareSolverrConfig,
    wordpress_api: WordPressApiConfig,
}

#[derive(Deserialize)]
struct BaseConfig {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct FileConfig {
    source_data: String,
    processing_data: String,
}

#[derive(Deserialize)]
struct FlareSolverrConfig {
    flaresolverr_url: String,
}

#[derive(Deserialize)]
struct WordPressApiConfig {
    wordpress_url: String,
    username_api: String,
    password_api: String,
}

fn load_config() -> Result<AppConfig, ConfigError> {
    let mut settings = Config::builder()
        .add_source(File::new("Settings.toml", config::FileFormat::Toml))
        .add_source(Environment::with_prefix("APP"))
        .build()?
        .try_deserialize::<AppConfig>()?;

    Ok(settings)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    println!("Application Name: {}", config.base.name);

    let root_id_page = 6559;
    // Setup CSV reading
    let file = AsyncFile::open(&config.file.source_data).await?;
    let reader = BufReader::new(file);
    let mut csv_reader = AsyncReaderBuilder::new()
        .has_headers(true)
        .create_reader(reader);

    // Attempt to read headers
    let headers = csv_reader.headers().await?;

    // Setup CSV writing
    let file_out = AsyncFile::create(&config.file.processing_data).await?;
    let writer = BufWriter::new(file_out);
    let mut csv_writer = AsyncWriterBuilder::new()
        .delimiter(b'|')
        .quote(b'"')
        .double_quote(true)
        .create_writer(writer);

    // Write headers to the new CSV
    csv_writer.write_record(headers).await?;

    let client = Client::new();

    let mut records = csv_reader.records();

    // Processing records
    while let Some(record) = records.next().await {
        match record {
            Ok(record) => {
                let url = record.get(0).unwrap_or_default(); // Get the URL from record

                // Prepare the data for FlareSolverr request
                let data = json!({
                    "cmd": "request.get",
                    "url": url,
                    "maxTimeout": 60000
                });

                // Send the request to FlareSolverr
                let response = client
                    .post(&config.flaresolverr.flaresolverr_url)
                    .header(header::CONTENT_TYPE, "application/json")
                    .json(&data)
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let body: FlareSolverrResponse = resp.json().await?;

                            let ps_url = &body.solution.url;
                            let title = extract_title::extract_title(&body.solution.response);
                            let product_id =
                                extract_product_id::extract_product_id(&body.solution.response);
                            let price_ht =
                                extract_price_ht::extract_price_ht(&body.solution.response);

                            match product_id {
                                Some(value) => println!("product_id: {}", value),
                                None => println!("product_id: {}", ""),
                            }

                            let breadcrumbs =
                                extract_breadcrumb::extract_breadcrumb(&body.solution.response);

                            println!("ps_url: {}", ps_url);
                            println!("title: {}", title);

                            match price_ht {
                                Some(price) => println!("price_ht: {}", price),
                                None => println!("price_ht: {}", ""),
                            }

                            // Create page to Wordpress
                            let content = "<p>This is a test page</p>";
                            let status = "draft";
                            let wordpress_url = &config.wordpress_api.wordpress_url;
                            let username = &config.wordpress_api.username_api;
                            let password = &config.wordpress_api.password_api;
                            let mut current_parent_id = root_id_page;

                            for breadcrumb in &breadcrumbs {
                                if let Some(name) = breadcrumb.get("name") {
                                    println!("Nom du Breadcrumb: {}", name);
                                    // Appel de la fonction pour créer une page WordPress avec le parent ID actuel
                                    match create_wordpress_page::create_wordpress_page(
                                        name, // Utilisation du nom du breadcrumb pour le titre de la page
                                        content,
                                        status,
                                        wordpress_url,
                                        username,
                                        password,
                                        current_parent_id, // Utilisation de l'ID parent actuel
                                    )
                                    .await
                                    {
                                        Ok(response) => {
                                            println!("Page created successfully: {}", response);
                                            // Traitement de la réponse JSON pour extraire l'ID de la page créée
                                            match serde_json::from_str::<serde_json::Value>(
                                                &response,
                                            ) {
                                                Ok(json_value) => {
                                                    if let Some(id) = json_value
                                                        .get("id")
                                                        .and_then(|v| v.as_i64())
                                                    {
                                                        current_parent_id = id as i32; // Mise à jour du parent_id pour le prochain breadcrumb
                                                        println!(
                                                            "ID de la nouvelle page: {}",
                                                            current_parent_id
                                                        );
                                                    } else {
                                                        eprintln!("L'ID de la page n'a pas pu être extrait de la réponse.");
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Erreur lors de l'analyse de la réponse JSON: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => eprintln!("Error creating page: {:?}", e),
                                    }
                                }
                            }

                        //csv_writer.write_record(&[url, &title, ""]).await?;
                        } else {
                            let status = resp.status().to_string();
                            csv_writer
                                .write_record(&[url, &status, "Request failed"])
                                .await?;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to process URL {}: {}", url, e);
                        csv_writer
                            .write_record(&[url, "Failed to send request", ""])
                            .await?;
                    }
                }
            }
            Err(e) => eprintln!("Error reading CSV: {}", e),
        }
        time::sleep(Duration::from_secs(0)).await;
    }

    Ok(())
}
