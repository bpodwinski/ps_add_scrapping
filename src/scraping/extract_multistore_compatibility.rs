use scraper::{ElementRef, Html, Selector};

pub fn extract_multistore_compatibility(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Trouver le div qui contient le texte 'Compatibilité multiboutique'
    for element in document.select(&selector) {
        if element.text().any(|t| t.contains("Compatibilité multiboutique")) {
            // Essayer de naviguer au prochain élément frère qui est un élément du DOM
            let mut next_node = element.next_sibling();
            while let Some(node) = next_node {
                if let Some(element) = ElementRef::wrap(node) {
                    // Extraire et retourner le texte du premier élément frère trouvé
                    return element.text().collect::<Vec<_>>().join("").trim().to_string();
                }
                next_node = node.next_sibling();
            }
            break;
        }
    }

    // Retourner une chaîne vide si aucun élément approprié n'est trouvé
    String::new()
}
