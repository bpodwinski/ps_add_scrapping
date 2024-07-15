use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{from_str, json, Value};

use crate::wordpress::main::{Auth, CreateProduct};

#[derive(Debug)]
pub struct ProductCreationResult {
    pub http_status: u16,
    pub response_body: String,
    pub response_json: Value,
}

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
        images: &Vec<String>,
        ps_product_id: u32,
        ps_product_url: String,
    ) -> Result<ProductCreationResult> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let create_url = format!("{}/wp-json/wc/v3/products", self.base_url);

        let product = json!({
            "name": name,
            "type": r#type,
            "status": status,
            "virtual": r#virtual,
            "downloadable": downloadable,
            "short_description": short_description,
            "description": description,
            "regular_price": regular_price,
            "categories": categories.iter().map(|&id| json!({ "id": id })).collect::<Vec<_>>(),
            "images": images.iter().map(|url| json!({ "src": url })).collect::<Vec<_>>(),
            "meta_data": [
            {
              "key": "ps_product_id",
              "value": ps_product_id.to_string()
            },
            {
              "key": "ps_product_url",
              "value": ps_product_url
            }
          ]
        });

        let response = client
            .post(&create_url)
            .headers(headers)
            .json(&product)
            .send()
            .await
            .context("Failed to send create product request")?;

        // Save the status before consuming the response
        let status_code = response.status().as_u16();
        let response_body = response
            .text()
            .await
            .context("Failed to read response body")?;

        let body_json: Result<Value, _> = from_str(&response_body);

        let result = ProductCreationResult {
            http_status: status_code,
            response_body: response_body.clone(),
            response_json: body_json.unwrap_or(json!({"raw_body": response_body})),
        };

        match status_code {
            200 | 201 => Ok(result),
            _ => Err(anyhow::anyhow!(format!(
                "HTTP {}: {}",
                status_code, response_body
            ))),
        }
    }
}
