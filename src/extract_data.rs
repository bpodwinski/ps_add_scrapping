use std::collections::HashMap;

use crate::config::config::FlareSolverrResponse;
use crate::scraping::{extract_breadcrumb, extract_price_ht, extract_product_id, extract_title};

// Extract data scraped from server flaresolverr
pub fn extract_data(body: &FlareSolverrResponse) -> Vec<HashMap<String, String>> {
    // Extract data scraped from server flaresolverr
    let ps_url = &body.solution.url;
    let title = extract_title::extract_title(&body.solution.response);
    let product_id =
        extract_product_id::extract_product_id(&body.solution.response);
    let price_ht =
        extract_price_ht::extract_price_ht(&body.solution.response);
    let breadcrumbs =
        extract_breadcrumb::extract_breadcrumb(&body.solution.response);

    match product_id {
        Some(value) => println!("Product ID: {}", value),
        None => println!("Product ID: {}", ""),
    }

    match price_ht {
        Some(price) => println!("Price HT: {}", price),
        None => println!("Price HT: {}", ""),
    }

    println!("Title: {}", title);
    println!("PS Url: {}", ps_url);

    breadcrumbs
}
