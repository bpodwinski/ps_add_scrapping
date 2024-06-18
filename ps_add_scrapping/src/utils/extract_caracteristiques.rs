use html5ever::serialize::{serialize, SerializeOpts};
use scraper::node::Element;
use scraper::{Html, Selector};

pub fn extract_caracteristiques(html_content: &str) -> Option<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("div.product-description__content").unwrap();

    // Trouver la deuxième div correspondante et extraire son contenu HTML interne
    document
        .select(&selector)
        .nth(1) // Utilise nth pour sélectionner la deuxième div (indexé à partir de 0)
        .map(|element| {
            let mut inner_html = String::new();
            for child in element.children() {
                if let Some(element) = child.value().as_element() {
                    let mut bytes = Vec::new();
                    let serializer_opts = SerializeOpts {
                        traversal_scope: html5ever::serialize::TraversalScope::IncludeNode,
                        ..Default::default()
                    };
                    serialize(&mut bytes, &Element::as_node(element), serializer_opts)
                        .expect("Unable to serialize HTML element.");
                    inner_html.push_str(&String::from_utf8(bytes).expect("Found invalid UTF-8"));
                }
            }
            inner_html.replace("<div>", "").replace("</div>", "")
        })
}
