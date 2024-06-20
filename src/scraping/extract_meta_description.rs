use scraper::{Html, Selector};

pub fn get_meta_description(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("meta[name='description']").unwrap();

    // Sélectionner la première balise meta avec l'attribut name='description' et extraire la valeur de l'attribut 'content'
    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("content").map(String::from))
}
