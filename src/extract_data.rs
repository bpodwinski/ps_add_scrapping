use std::collections::HashMap;

use crate::config::config::FlareSolverrResponse;
use crate::scraping::{extract_breadcrumb, extract_developer_name, extract_price_ht, extract_product_id, extract_title};

#[derive(Debug)]
pub struct ScrapedData {
    pub breadcrumbs: Vec<HashMap<String, String>>,
    pub product_id: String,
    pub price_ht: String,
    pub title: String,
    pub developer_name: String,
    pub ps_url: String,
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

    ScrapedData {
        breadcrumbs,
        product_id,
        price_ht,
        title,
        developer_name,
        ps_url,
    }
}
