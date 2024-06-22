use scraper::{Html, Selector};
use scraper::element_ref::ElementRef;

pub fn extract_override(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let title_selector = Selector::parse("div.muik-section-item__title.puik-body-small").unwrap();

    // Trouver le div spécifique contenant le texte 'Contient des surcharges'
    if let Some(title_div) = document.select(&title_selector).find(|element| {
        element.text().any(|text| text.contains("Contient des surcharges"))
    }) {
        // Tenter de naviguer au prochain div qui contiendrait la date
        let mut next_node = title_div.next_sibling();
        while let Some(node) = next_node {
            if let Some(element) = ElementRef::wrap(node) {
                // Extraire et retourner la date
                return element.text().collect::<Vec<_>>().join("").trim().to_string();
            }
            next_node = node.next_sibling();
        }
        println!("No valid following div found containing the date.");
    } else {
        println!("No div containing 'Contient des surcharges' title found.");
    }

    // Retourner une chaîne vide si aucune date valide n'est trouvée
    String::new()
}
