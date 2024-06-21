use std::collections::HashMap;

use crate::config::config::FlareSolverrResponse;
use crate::scraping::{
    extract_breadcrumb,
    extract_caracteristiques,
    extract_description,
    extract_developer_name,
    extract_last_update,
    extract_module_version,
    extract_multistore_compatibility,
    extract_override,
    extract_price_ht,
    extract_product_id,
    extract_publication_date,
    extract_title,
};

#[derive(Debug)]
pub struct ScrapedData {
    pub breadcrumbs: Vec<HashMap<String, String>>,
    pub product_id: String,
    pub price_ht: String,
    pub title: String,
    pub developer_name: String,
    pub ps_url: String,
    pub module_version: String,
    pub last_update: String,
    pub multistore_compatibility: String,
    pub publication_date: String,
    pub caracteristiques: String,
    pub with_override: String,
    pub description: String,
}

// Extract data scraped from server flaresolverr
pub fn extract_data(body: &FlareSolverrResponse) -> ScrapedData {

    // Extract data scraped from server flaresolverr
    let ps_url = body.solution.url.clone();
    let title = extract_title::extract_title(&body.solution.response);
    let product_id =
        extract_product_id::extract_product_id(&body.solution.response);
    let price_ht =
        extract_price_ht::extract_price_ht(&body.solution.response);
    let developer_name =
        extract_developer_name::extract_developer_name(&body.solution.response);
    let breadcrumbs =
        extract_breadcrumb::extract_breadcrumb(&body.solution.response);
    let module_version = extract_module_version::extract_module_version(&body.solution.response);
    let last_update = extract_last_update::extract_last_update(&body.solution.response);
    let multistore_compatibility = extract_multistore_compatibility::extract_multistore_compatibility(&body.solution.response);
    let publication_date = extract_publication_date::extract_publication_date(&body.solution.response);
    let caracteristiques = extract_caracteristiques::extract_caracteristiques(&body.solution.response);
    let with_override = extract_override::extract_override(&body.solution.response);
    let description = extract_description::extract_description(&body.solution.response);

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
        caracteristiques,
        with_override,
        description,
    }
}
