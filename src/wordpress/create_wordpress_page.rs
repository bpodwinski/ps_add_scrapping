use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use reqwest::Client;
use serde_json::json;

pub async fn create_wordpress_page(
    title: &str,
    content: &str,
    product_id: &str,
    status: &str,
    author: i32,
    wordpress_url: &str,
    username: &str,
    password: &str,
    parent: i32,
) -> Result<String> {
    let client = Client::new();
    let credentials = format!("{}:{}", username, password);
    let engine = STANDARD_NO_PAD;
    let encoded_credentials = engine.encode(credentials.as_bytes());
    let basic_auth = format!("Basic {}", encoded_credentials);

    let create_url = format!("{}/wp-json/wp/v2/pages", wordpress_url);
    let page = json!({
        "post_type": "page",
        "title": title,
        "content": content,
        "meta": {
            "ps_product_id": product_id
        },
        "status": status,
        "author": author,
        "parent": parent
    });

    let response = client
        .post(&create_url)
        .header("Authorization", &basic_auth)
        .json(&page)
        .send()
        .await
        .context("Failed to send create page request")?;

    if !response.status().is_success() {
        let error_msg = format!("Failed to create page: {}", response.status());
        anyhow::bail!(error_msg);
    }

    let body = response
        .text()
        .await
        .context("Failed to read create page response text")?;
    Ok(body)
}
