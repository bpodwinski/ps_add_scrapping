use scraper::{ElementRef, Html, Selector};

pub fn extract_publication_date(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Find the div containing the text 'Date de publication'
    let target_div = document.select(&selector).find(|element| {
        element.text().any(|text| text.contains("Date de publication"))
    });

    if let Some(div) = target_div {
        // Attempt to navigate to the next sibling that is an element node
        let mut next_node = div.next_sibling();
        while let Some(node) = next_node {
            if let Some(element) = ElementRef::wrap(node) {
                // Return the text of the first sibling element found
                return element.text().collect::<Vec<_>>().join("").trim().to_string();
            }
            next_node = node.next_sibling();
        }
        println!("No valid following div found.");
    } else {
        println!("No div containing 'Date de publication' found.");
    }

    // Return empty string if no valid date is found
    String::new()
}
