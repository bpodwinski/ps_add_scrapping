use scraper::{Html, Selector};

pub fn extract_caracteristiques(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div.product-description__content").unwrap();

    if let Some(element) = document.select(&selector).nth(1) {
        // Collecter tout le contenu textuel des enfants directement
        let inner_html = element.inner_html();
        // Nettoyer les balises div superflues s'il y en a
        inner_html.replace("<div>", "").replace("</div>", "")
    } else {
        // Retourner une chaîne vide si aucune div valide n'est trouvée
        String::new()
    }
}
