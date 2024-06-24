use regex::Regex;
use scraper::{Html, Selector};

/// Extracts the product ID from the HTML content as a string, returns empty string if not found.
pub fn extract_product_id(html_content: &str) -> u32 {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("script").unwrap();
    let regex = Regex::new(r#""id_product":(\d+)"#).unwrap();

    // Iterate over script elements and apply regex to find id_product
    document.select(&selector).filter_map(|script| {
        script.text().find_map(|text| {
            regex.captures(text).and_then(|caps| {
                caps.get(1).map(|match_| match_.as_str().to_string())
            })
        })
    }).next().unwrap_or_default().parse().unwrap()
}
