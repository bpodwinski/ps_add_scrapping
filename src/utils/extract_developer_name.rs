use scraper::{Html, Selector};

pub fn extract_developer_name(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("a[id='ps_link_manufacturer']").unwrap();

    // Sélectionner le premier lien avec l'ID spécifié et extraire l'attribut 'title'
    document.select(&selector).next().map(|element| {
        element
            .value()
            .attr("title")
            .unwrap_or_default()
            .to_string()
    })
}
