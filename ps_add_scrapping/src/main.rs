use csv_async::{AsyncReaderBuilder, AsyncWriterBuilder};
use futures::stream::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use serde_json::json;
use tokio::fs::File;
use tokio::io::{BufReader, BufWriter};
use tokio::time::{self, Duration};

// Utils import
mod utils;
use utils::create_wordpress_page;
use utils::extract_breadcrumb;
use utils::extract_caracteristiques;
use utils::extract_description;
use utils::extract_developer_name;
use utils::extract_last_update;
use utils::extract_meta_description;
use utils::extract_module_version;
use utils::extract_multistore_compatibility;
use utils::extract_override;
use utils::extract_price_ht;
use utils::extract_product_id;
use utils::extract_publication_date;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup CSV reading
    let file = File::open("***").await?;
    let reader = BufReader::new(file);
    let mut csv_reader = AsyncReaderBuilder::new()
        .has_headers(true)
        .create_reader(reader);

    // Attempt to read headers
    let headers = csv_reader.headers().await?;

    // Setup CSV writing
    let file_out = File::create("***").await?;
    let writer = BufWriter::new(file_out);
    let mut csv_writer = AsyncWriterBuilder::new()
        .delimiter(b'|')
        .quote(b'"')
        .double_quote(true)
        .create_writer(writer);

    // Write headers to the new CSV
    csv_writer.write_record(headers).await?;

    let client = Client::new();
    let flaresolverr_url = "***";

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
                    .post(flaresolverr_url)
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

                            let breadcrumb =
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
                            let wordpress_url = "https://artisanwebmaster.com";
                            let username = "flare_api";
                            let password = "e1G2 4nN8 5id9 ZOSA z7KK LmSP";

                            match create_wordpress_page::create_wordpress_page(
                                &title,
                                content,
                                status,
                                wordpress_url,
                                username,
                                password,
                            )
                            .await
                            {
                                Ok(response) => {
                                    println!("Page created successfully: {:?}", response)
                                }
                                Err(e) => eprintln!("Error creating page: {:?}", e),
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
