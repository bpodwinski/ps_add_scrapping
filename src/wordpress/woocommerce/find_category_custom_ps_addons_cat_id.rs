use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::wordpress::main::{Auth, FindCategoryCustomPsAddonsCatId};

pub struct CategoryInfo {
    pub status: String,
    pub message: String,
    pub category_id: Option<u32>,
    pub ps_addons_cat_id: Option<u32>,
    pub category_name: Option<String>,
}

impl FindCategoryCustomPsAddonsCatId for Auth {
    async fn find_category_custom_ps_addons_cat_id(&self, ps_addons_cat_id: u32) -> Result<CategoryInfo> {
        let client = Client::new();
        let headers = self.create_headers(None)?;

        let api_url = format!(
            "{}/wp-json/wc/v3/products/categories?ps_addons_cat_id={}",
            self.base_url(), ps_addons_cat_id
        );

        let response = client
            .get(api_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to send search request for category")?;

        if response.status().is_success() {
            let categories: Vec<Value> = response
                .json()
                .await
                .context("Failed to parse search response for category as JSON")?;

            if !categories.is_empty() {
                for category in categories {
                    if category["ps_addons_cat_id"] == ps_addons_cat_id {
                        return Ok(CategoryInfo {
                            status: "found".to_string(),
                            message: "Category already exists".to_string(),
                            category_id: category["id"].as_u64().map(|id| id as u32),
                            ps_addons_cat_id: Some(ps_addons_cat_id),
                            category_name: category["name"].as_str().map(|name| name.to_string()),
                        });
                    }
                }

                return Ok(CategoryInfo {
                    status: "notfound".to_string(),
                    message: "No category found with the given ID".to_string(),
                    category_id: None,
                    ps_addons_cat_id: None,
                    category_name: None,
                });
            }
        } else {
            let error_msg = format!("Failed to search for category: {}", response.status());
            anyhow::bail!(error_msg);
        }
        Ok(CategoryInfo {
            status: "error".to_string(),
            message: "Unknown error occurred".to_string(),
            category_id: None,
            ps_addons_cat_id: None,
            category_name: None,
        })
    }
}
