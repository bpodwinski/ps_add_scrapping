use std::collections::HashMap;

use serde::Deserialize;

use crate::extractors::ps_addons::{
    extract_breadcrumb, extract_description, extract_developer_name, extract_features,
    extract_image_urls, extract_last_update, extract_module_version,
    extract_multistore_compatibility, extract_override, extract_price_ht, extract_product_id,
    extract_ps_version_required, extract_publication_date, extract_title,
};

#[derive(Debug, Deserialize)]
pub struct FlareSolverrResponse {
    pub solution: Solution,
    pub status: String,
    pub message: String,
    #[serde(rename = "startTimestamp")]
    pub start_timestamp: u64,
    #[serde(rename = "endTimestamp")]
    pub end_timestamp: u64,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct Solution {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub response: String,
    pub cookies: Vec<Cookie>,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
}

#[derive(Debug, Deserialize)]
pub struct Cookie {
    pub name: Option<String>,
    pub value: Option<String>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<f64>,
    pub size: Option<usize>,
    #[serde(rename = "httpOnly")]
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub session: Option<bool>,
    #[serde(rename = "sameSite")]
    pub same_site: Option<String>,
}

#[derive(Debug)]
pub struct ScrapedData {
    pub breadcrumbs: Vec<HashMap<String, String>>,
    pub product_id: u32,
    pub price_ht: String,
    pub title: String,
    pub developer_name: String,
    pub ps_url: String,
    pub module_version: String,
    pub last_update: String,
    pub multistore_compatibility: String,
    pub publication_date: String,
    pub features: String,
    pub with_override: String,
    pub description: String,
    pub ps_version_required: String,
    pub image_urls: Vec<String>,
}

// Extract data scraped from server flaresolverr
pub fn extract_data(body: &FlareSolverrResponse) -> ScrapedData {
    // Extract data
    let ps_url = body.solution.url.clone();
    let title = extract_title::extract_title(&body.solution.response);
    let product_id = extract_product_id::extract_product_id(&body.solution.response);
    let price_ht = extract_price_ht::extract_price_ht(&body.solution.response);
    let developer_name = extract_developer_name::extract_developer_name(&body.solution.response);
    let breadcrumbs = extract_breadcrumb::extract_breadcrumb(&body.solution.response);
    let module_version = extract_module_version::extract_module_version(&body.solution.response);
    let last_update = extract_last_update::extract_last_update(&body.solution.response);
    let multistore_compatibility =
        extract_multistore_compatibility::extract_multistore_compatibility(&body.solution.response);
    let publication_date =
        extract_publication_date::extract_publication_date(&body.solution.response);
    let features = extract_features::extract_features(&body.solution.response);
    let with_override = extract_override::extract_override(&body.solution.response);
    let description = extract_description::extract_description(&body.solution.response);
    let ps_version_required =
        extract_ps_version_required::extract_ps_version_required(&body.solution.response);

    // Extract urls images
    let base_url = "https://addons.prestashop.com/";
    let image_urls = extract_image_urls::extract_image_urls(&body.solution.response, base_url);

    ScrapedData {
        breadcrumbs,
        product_id,
        price_ht,
        title,
        developer_name,
        ps_url,
        module_version,
        last_update,
        multistore_compatibility,
        publication_date,
        features,
        with_override,
        description,
        ps_version_required,
        image_urls,
    }
}
