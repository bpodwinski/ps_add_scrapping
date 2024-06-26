use scraper::{Html, Selector};

/// Extract image URLs from HTML content that contain a specific base URL.
pub fn extract_image_urls(html_content: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("img").unwrap();
    let mut images_url = Vec::new();

    for element in document.select(&selector) {
        if let Some(src) = element.value().attr("src") {
            if src.contains(base_url) {
                images_url.push(src.to_string());
                println!("Link found in src: {}", src);
            }
        }
    }
    images_url
}
