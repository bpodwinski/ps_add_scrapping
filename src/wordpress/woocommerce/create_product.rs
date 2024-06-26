use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::wordpress::main::{Auth, CreateProduct};

impl CreateProduct for Auth {
    async fn create_product(
        &self,
        name: String,
        status: String,
        r#type: String,
        r#virtual: bool,
        downloadable: bool,
        short_description: String,
        description: String,
        regular_price: String,
        categories: Vec<u32>,
        images: Vec<String>,
    ) -> Result<String> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let create_url = format!("{}/wp-json/wc/v3/products", self.base_url);

        let product = json!({
            "name": name,
            "type": r#type,
            "status": status,
            "r#virtual": r#virtual,
            "downloadable": downloadable,
            "short_description": short_description,
            "description": description,
            "regular_price": regular_price,
            "categories": categories.iter().map(|&id| json!({ "id": id })).collect::<Vec<_>>(),
            "images": images.iter().map(|url| json!({ "src": url })).collect::<Vec<_>>()
        });

        let response = client
            .post(&create_url)
            .headers(headers)
            .json(&product)
            .send()
            .await
            .context("Failed to send create product request")?;

        let status_code = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;

        match status_code {
            StatusCode::CREATED => Ok(response_body),
            StatusCode::BAD_REQUEST => Err(anyhow::anyhow!(response_body)),
            _ => Err(anyhow::anyhow!("Failed to create product with status {}: {}", status_code, response_body))
        }
    }
}
