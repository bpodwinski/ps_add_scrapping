use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use reqwest::{Client, Error};
use serde_json::json;

pub async fn create_wordpress_page(
    title: &str,
    content: &str,
    status: &str,
    wordpress_url: &str,
    username: &str,
    password: &str,
    parent_id: i32,
) -> Result<String, Error> {
    let client = Client::new();
    let credentials = format!("{}:{}", username, password);
    let engine = STANDARD_NO_PAD;
    let encoded_credentials = engine.encode(credentials.as_bytes());
    let basic_auth = format!("Basic {}", encoded_credentials);

    // Vérifier si la page existe déjà
    let search_url = format!("{}/wp-json/wp/v2/pages?search={}", wordpress_url, title);
    let search_response = client
        .get(&search_url)
        .header("Authorization", basic_auth.clone())
        .send()
        .await?;

    if search_response.status().is_success() {
        let search_body = search_response.text().await?;
        if !search_body.is_empty() {
            return Ok("Page already exists".to_string());
        }
    }

    // Créer la page si elle n'existe pas
    let create_url = format!("{}/wp-json/wp/v2/pages", wordpress_url);
    let page = json!({
        "post_type": "page",
        "title": title,
        "content": content,
        "status": status,
        "post_author": "1",
        "parent": parent_id
    });

    let response = client
        .post(&create_url)
        .header("Authorization", basic_auth)
        .json(&page)
        .send()
        .await?;

    let body = response.text().await?;
    //println!("Raw response body: {}", body);
    Ok(body)
}
