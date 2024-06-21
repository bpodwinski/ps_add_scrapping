use scraper::{Html, Selector};

pub fn extract_description(html_content: &str) -> String {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div.product-description__content").unwrap();

    // Sélectionner l'élément et récupérer son contenu HTML interne
    if let Some(element) = document.select(&selector).next() {
        let inner_html = element.inner_html();
        // Supprimer les balises div inutiles et retourner le HTML nettoyé
        inner_html.replace("<div>", "").replace("</div>", "")
    } else {
        // Retourner une chaîne vide si aucun élément correspondant n'est trouvé
        String::new()
    }
}
