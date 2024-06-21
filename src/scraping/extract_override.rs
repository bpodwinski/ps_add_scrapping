use scraper::{ElementRef, Html, Selector};

pub fn extract_override(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Trouver le div contenant le texte spécifique "Contient des surcharges"
    let target_div = document.select(&selector).find(|element| {
        element.text().any(|text| text.contains("Contient des surcharges"))
    });

    if let Some(div) = target_div {
        // Tenter de naviguer au prochain élément frère qui est un nœud élément
        let mut next_node = div.next_sibling();
        while let Some(node) = next_node {
            if let Some(element) = ElementRef::wrap(node) {
                // Extraire et retourner le texte du premier élément frère trouvé
                return element.text().collect::<Vec<_>>().join("").trim().to_string();
            }
            next_node = node.next_sibling();
        }
        // Aucun frère suivant valide trouvé, retourner une chaîne vide
        println!("No valid following div found.");
    } else {
        // Aucun div contenant le texte recherché trouvé, retourner une chaîne vide
        println!("No div containing 'Contains overrides' found.");
    }

    // Retourner une chaîne vide si le contenu n'est pas trouvé
    String::new()
}
