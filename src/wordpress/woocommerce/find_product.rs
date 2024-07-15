use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::wordpress::main::{Auth, FindProductByCustomField};

pub struct ProductInfo {
    pub status: String,
    pub message: String,
    pub product_id: Option<u32>,
    pub custom_field_value: Option<String>,
}

impl FindProductByCustomField for Auth {
    async fn find_product_by_custom_field(
        &self,
        custom_field_key: &str,
        custom_field_value: &str,
    ) -> Result<ProductInfo> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let api_url = format!(
            "{}/wp-json/wc/v3/products?{}={}",
            self.base_url(),
            custom_field_key,
            custom_field_value
        );

        let response = client
            .get(&api_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to send request to WooCommerce API")?;

        if response.status().is_success() {
            let products: Vec<Value> = response
                .json()
                .await
                .context("Failed to parse response as JSON")?;

            for product in products {
                if let Some(meta_data) = product["meta_data"].as_array() {
                    for meta in meta_data {
                        if meta["key"] == custom_field_key && meta["value"] == custom_field_value {
                            return Ok(ProductInfo {
                                status: "found".to_string(),
                                message: "Product already exists".to_string(),
                                product_id: product["id"].as_u64().map(|id| id as u32),
                                custom_field_value: Some(custom_field_value.to_string()),
                            });
                        }
                    }
                }
            }

            Ok(ProductInfo {
                status: "notfound".to_string(),
                message: "No product found with the given custom field value".to_string(),
                product_id: None,
                custom_field_value: None,
            })
        } else {
            let error_msg = format!("Failed to search for product: {}", response.status());
            anyhow::bail!(error_msg);
        }
    }
}
