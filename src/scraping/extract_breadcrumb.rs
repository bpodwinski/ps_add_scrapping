use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub fn extract_breadcrumb(response_html: &str) -> Vec<HashMap<String, String>> {
    let document = Html::parse_document(response_html);
    let selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let mut breadcrumbs = Vec::new();

    for element in document.select(&selector) {
        let script_content = element.text().collect::<Vec<_>>().join("");
        let data: Value = serde_json::from_str(&script_content).unwrap_or(Value::Null);

        if let Value::Object(obj) = data {
            if obj.get("@type") == Some(&Value::String("BreadcrumbList".to_string())) {
                if let Some(item_list) = obj.get("itemListElement") {
                    if let Value::Array(items) = item_list {
                        for item in items {
                            if let Value::Object(item_obj) = item {
                                let position = item_obj
                                    .get("position")
                                    .and_then(|v| v.as_i64())
                                    .map(|v| v.to_string())
                                    .unwrap_or_default();

                                let id = item_obj
                                    .get("item")
                                    .and_then(|v| v.get("@id"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string();

                                let name = item_obj
                                    .get("item")
                                    .and_then(|v| v.get("name"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string();

                                let mut crumb = HashMap::new();
                                crumb.insert("position".to_string(), position);
                                crumb.insert("id".to_string(), id);
                                crumb.insert("name".to_string(), name);
                                breadcrumbs.push(crumb);
                            }
                        }
                    }
                }
                break; // Quit the loop once breadcrumbs are found and processed
            }
        }
    }
    breadcrumbs
}
