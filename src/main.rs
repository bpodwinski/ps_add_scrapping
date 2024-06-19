use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio;
use tokio::fs::File as AsyncFile;
use tokio::io::{BufReader, BufWriter};

// Import modules
mod config;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = match config::load_config() {
        Ok(cfg) => cfg, // Si le chargement réussit, stocke la configuration dans la variable config
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // Setup CSV reading
    let file = AsyncFile::open(&config.file.source_data).await?;
    let reader = BufReader::new(file);
    let mut csv_reader = AsyncReaderBuilder::new().create_reader(reader);

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
    let max_concurrency = 1;

    // Utilise directement le stream des enregistrements
    let records_stream = csv_reader.records();

    let flaresolverr_url = &config.flaresolverr.flaresolverr_url;
    let wordpress_url = &config.wordpress_api.wordpress_url;
    let username = &config.wordpress_api.username_api;
    let password = &config.wordpress_api.password_api;

    // Traitement des URLs avec une concurrence limitée
    records_stream
        .for_each_concurrent(max_concurrency, |record_result| {
            let client = client.clone(); // Cloner le client pour l'usage dans l'async block
            
            async move {
                match record_result {
                    Ok(record) => {
                        
                        let url = record.get(0).unwrap_or_default(); // Assure-toi que l'index est correct
                        let data = json!({
                            "cmd": "request.get",
                            "url": url,
                            "maxTimeout": 60000
                        });

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
                                let mut current_parent_id = config.base.root_id_page;

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
                                                println!("Page created successfully");
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
                            } else {
                                eprintln!("Failed to deserialize response");
                            }

                            } else {
                                }
                            } else {
                                // Logique pour gérer les erreurs de requête
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read CSV record: {}", e);
                        }
                    }
                }
            })
            .await;
    Ok(())
}
