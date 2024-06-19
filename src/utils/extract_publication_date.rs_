use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};

pub fn extract_publication_date(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Find the div containing the specific text 'Date de publication'
    let target_div = document.select(&selector).find(|element| {
        element
            .text()
            .any(|text| text.contains("Date de publication"))
    });

    if let Some(div) = target_div {
        // Attempt to navigate to the next sibling that is an element node
        let mut next_element = div.next_sibling();
        while let Some(sibling) = next_element {
            if sibling.value().is_element() {
                return sibling
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string()
                    .into();
            }
            next_element = sibling.next_sibling();
        }
        println!("Aucune div suivante trouvée ou valide.");
    } else {
        println!("Aucune div contenant 'Date de publication' trouvée.");
    }

    None
}
