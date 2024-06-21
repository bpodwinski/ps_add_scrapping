use scraper::{Html, Selector};

pub fn extract_module_version(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("span.module__title-version").unwrap();

    // Select the first span with the specified class and extract its textual content
    document
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<Vec<_>>().join(""))
        .unwrap_or_default() // Returns an empty string if None
}
