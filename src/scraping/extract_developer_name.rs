use scraper::{Html, Selector};

/// Extracts the developer name from HTML content
pub fn extract_developer_name(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("a[id='ps_link_manufacturer']").unwrap();

    // Select the first link with the specified ID and extract the 'title' attribute
    document.select(&selector).next().map_or_else(
        || String::new(), // Return an empty string if no element is found
        |element| element.value().attr("title").unwrap_or_default().to_string(),
    )
}
