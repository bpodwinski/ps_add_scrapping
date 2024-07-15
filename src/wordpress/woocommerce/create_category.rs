use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

use crate::wordpress::main::{Auth, CreateCategory};

impl CreateCategory for Auth {
    async fn create_category(
        &self,
        name: String,
        parent: u32,
        ps_addons_cat_id: u32,
    ) -> Result<Value> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let create_url = format!("{}/wp-json/wc/v3/products/categories", self.base_url);

        let category = json!({
            "name": name,
            "parent": parent,
            "ps_addons_cat_id": ps_addons_cat_id,
        });

        let response = client
            .post(&create_url)
            .headers(headers)
            .json(&category)
            .send()
            .await
            .context("Failed to send create category request")?;

        let status_code = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;

        let response_json: Value = serde_json::from_str(&response_body)
            .context("Failed to parse response body as JSON")?;

        // TODO: change to return http_code
        match status_code {
            StatusCode::CREATED => Ok(response_json),
            StatusCode::BAD_REQUEST => Err(anyhow::anyhow!(response_body)),
            _ => Err(anyhow::anyhow!("Failed to create category with status {}: {}", status_code, response_body))
        }
    }
}