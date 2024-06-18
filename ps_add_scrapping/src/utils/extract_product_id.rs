use regex::Regex;
use scraper::{Html, Selector};

/// Extracts the product ID from the HTML content as an integer.
pub fn extract_product_id(html_content: &str) -> Option<i32> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("script").unwrap();
    let regex = Regex::new(r#""id_product":(\d+)"#).unwrap();

    // Iterate over script elements and apply regex to find id_product
    document.select(&selector).find_map(|script| {
        script
            .text()
            .find(|text| regex.is_match(text))
            .and_then(|script_content| {
                regex.captures(script_content).and_then(|caps| {
                    caps.get(1)
                        .and_then(|match_| match_.as_str().parse::<i32>().ok())
                })
            })
    })
}
