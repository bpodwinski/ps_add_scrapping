use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::{from_str, json, Value};

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
        images: &Vec<String>,
        ps_product_id: u32,
        ps_product_url: String,
    ) -> Result<Value> {
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
              "value": ps_product_id
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
        let status_code = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;

        let body_json: Result<Value, _> = from_str(&response_body);

        let result = json!({
            "http_status": status_code.as_u16(),
            "body": body_json.unwrap_or(json!({"raw_body": response_body})),
        });

        match status_code {
            StatusCode::CREATED => Ok(result),
            StatusCode::BAD_REQUEST => Ok(result),
            _ => Err(anyhow::anyhow!("Failed to create product with status {}: {}", status_code, response_body))
        }
    }
}
