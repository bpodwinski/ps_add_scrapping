use regex::Regex;

/// Extracts the price HT from the HTML content as a string, returns empty string if not found.
pub fn extract_price_ht(html_content: &str) -> String {
    let regex = Regex::new(r#""price":(\d+(\.\d+)?)?"#).unwrap();

    regex
        .captures(html_content)
        .and_then(|caps| caps.get(1).map(|match_| match_.as_str().to_string()))
        .unwrap_or_default()  // Return an empty string if no price is found
}
