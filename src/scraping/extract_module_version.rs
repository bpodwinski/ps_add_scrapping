use scraper::{Html, Selector};

pub fn get_module_version(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("span.module__title-version").unwrap();

    // Sélectionner le premier span avec la classe spécifiée et extraire son contenu textuel
    document
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<Vec<_>>().join(""))
}
