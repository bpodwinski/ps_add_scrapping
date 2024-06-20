use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};

pub fn extract_last_update(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Find the div containing the specific text 'Dernière mise à jour'
    let target_div = document.select(&selector).find(|element| {
        element
            .text()
            .any(|text| text.contains("Dernière mise à jour"))
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
        println!("Aucune div contenant 'Dernière mise à jour' trouvée.");
    }

    None
}
