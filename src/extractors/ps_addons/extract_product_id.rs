use regex::Regex;
use scraper::{Html, Selector};

/// Extracts the SKU (Stock Keeping Unit) from the HTML content by searching for the pattern `,"sku":<number>,`.
/// Returns 0 if no SKU is found.
pub fn extract_product_id(html_content: &str) -> u32 {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("script").unwrap();
    let regex = Regex::new(r#","sku":(\d+),"#).unwrap();

    // Iterate over script elements to find the script containing the SKU
    document
        .select(&selector)
        .find_map(|element| {
            element.text().find_map(|script_text| {
                // Apply regex to extract the SKU
                regex.captures(script_text).and_then(|caps| {
                    caps.get(1)
                        .map(|match_| match_.as_str().parse::<u32>().unwrap_or(0))
                })
            })
        })
        .unwrap_or(0) // Return 0 if no SKU is found
}
