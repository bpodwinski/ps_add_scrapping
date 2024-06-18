use scraper::node::Element;
use scraper::{Html, Selector};

pub fn extract_multistore_compatibility(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div").unwrap();

    // Trouver le div qui contient le texte 'Compatibilité multiboutique'
    let mut target_div = None;
    for element in document.select(&selector) {
        if element
            .text()
            .any(|t| t.contains("Compatibilité multiboutique"))
        {
            target_div = element.next_sibling();
            break;
        }
    }

    // Sauter les nœuds de texte vides pour trouver le nœud suivant significatif
    while let Some(node) = target_div {
        if node.value().is_element() {
            return Some(node.text().collect::<Vec<_>>().join(""));
        }
        target_div = node.next_sibling();
    }

    None
}
