use anyhow::{Context, Result};
use htmlentity::entity::{decode, ICodedDataTrait};
use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use urlencoding::encode;

use crate::wordpress::main::{Auth, FindPage};

impl FindPage for Auth {
    async fn find_page(&self, title: &str) -> Result<Option<String>> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let encoded_title = encode(title);
        let search_url = format!(
            "{}/wp-json/wp/v2/pages?search={}&status=draft,publish",
            self.base_url(), encoded_title
        );

        let response = client
            .get(search_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to send search request")?;

        if response.status().is_success() {
            let pages: Vec<Value> = response
                .json()
                .await
                .context("Failed to parse search response as JSON")?;

            let decoded_title = decode(title.as_bytes()).to_string()
                .context("Failed to decode the title for comparison")?;

            if !pages.is_empty() {
                for page in pages {
                    let found_title_encoded = page["title"]["rendered"].as_str().unwrap_or_default();
                    let found_title = decode(found_title_encoded.as_bytes()).to_string()?;

                    if found_title == decoded_title {
                        let json_response = json!({
                            "status": "exists",
                            "message": "Page already exists",
                            "page_id": page["id"],
                            "title": found_title
                        });
                        let json_string = serde_json::to_string(&json_response)
                            .with_context(|| "Failed to serialize response JSON")?;
                        return Ok(Some(json_string));
                    }
                }
            }
        } else {
            let error_msg = format!("Failed to search for page: {}", response.status());
            anyhow::bail!(error_msg);
        }
        Ok(None)
    }
}
