use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};

/// Extracts and returns the HTML content from the description section.
pub fn extract_description(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let title_selector = Selector::parse("div.product-description__title").unwrap();

    // Iterate over elements with the class 'product-description__title'
    for element in document.select(&title_selector) {
        // Check if the element contains a 'h2' with the text 'Description'
        if element.text().any(|text| text.contains("Description")) {
            // If found, traverse to the next element until we find 'product-description__content'
            let mut next_element = element.next_sibling();
            while let Some(next) = next_element {
                if let Some(next_ref) = ElementRef::wrap(next) {
                    // Check if this is the 'product-description__content'
                    if next_ref
                        .value()
                        .attr("class")
                        .map_or(false, |c| c.contains("product-description__content"))
                    {
                        // Return the raw HTML content from the 'product-description__content' div

                        // Nettoyer les balises div superflues s'il y en a
                        let inner_html = next_ref.inner_html();
                        return inner_html.replace("<div>", "").replace("</div>", "");
                    }
                }
                next_element = next.next_sibling();
            }
            break;
        }
    }

    // Return an empty string if no valid description is found
    "Aucun contenu de description valide trouvé".to_string()
}
