use regex::Regex;

pub fn extract_price_ht(html_content: &str) -> Option<String> {
    let regex = Regex::new(r#""price":(\d+(\.\d+)?)?"#).unwrap();

    regex
        .captures(html_content)
        .and_then(|caps| caps.get(1).map(|match_| match_.as_str().to_string()))
}
