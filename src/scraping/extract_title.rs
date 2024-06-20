use scraper::{Html, Selector};

pub fn extract_title(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("title").unwrap();

    document
        .select(&selector)
        .next()
        .map(|n| n.inner_html())
        .unwrap_or_else(|| "No title found".to_string())
}
