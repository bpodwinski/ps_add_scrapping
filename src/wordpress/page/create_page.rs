use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::wordpress::main::{Auth, CreatePage};

impl CreatePage for Auth {
    async fn create_page(
        &self,
        title: &str,
        content: &str,
        product_id: u32,
        product_url: &str,
        status: &str,
        author: u32,
        parent: u32,
    ) -> Result<String> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let create_url = format!("{}/wp-json/wp/v2/pages", self.base_url);

        let page = json!({
            "post_type": "page",
            "title": title,
            "content": content,
            "meta": {
                "ps_product_id": product_id,
                "ps_product_url": product_url
            },
            "status": status,
            "author": author,
            "parent": parent
        });

        let response = client
            .post(&create_url)
            .headers(headers)
            .json(&page)
            .send()
            .await
            .context("Failed to send create page request")?;

        // Save the status before consuming the response
        let status_code = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;

        match status_code {
            StatusCode::OK | StatusCode::CREATED => Ok(response_body),
            StatusCode::BAD_REQUEST => Err(anyhow::anyhow!(response_body)),
            _ => Err(anyhow::anyhow!("Failed to create page with status {}: {}", status_code, response_body))
        }
    }
}
