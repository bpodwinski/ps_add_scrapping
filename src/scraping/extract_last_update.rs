use scraper::{ElementRef, Html, Selector};

pub fn extract_last_update(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Find the div containing the specific text 'Dernière mise à jour'
    let target_div = document.select(&selector).find(|element| {
        element.text().any(|text| text.contains("Dernière mise à jour"))
    });

    if let Some(div) = target_div {
        // Attempt to navigate to the next sibling that is an element node
        let mut next_element = div.next_sibling();
        while let Some(sibling) = next_element {
            if let Some(element) = ElementRef::wrap(sibling) {
                // Now using ElementRef to access the text method
                return element.text().collect::<Vec<_>>().join("").trim().to_string();
            }
            next_element = sibling.next_sibling();
        }
        println!("No valid following div found.");
    } else {
        println!("No div containing 'Last update' found.");
    }

    // Return empty string if no div is found or if no valid sibling is found
    String::new()
}
