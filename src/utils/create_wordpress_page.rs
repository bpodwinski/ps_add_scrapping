use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use reqwest::{Client, Error, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use urlencoding::encode;

pub async fn create_wordpress_page(
    title: &str,
    content: &str,
    status: &str,
    wordpress_url: &str,
    username: &str,
    password: &str,
    parent_id: i32,
) -> Result<String> {
    let client = Client::new();
    let credentials = format!("{}:{}", username, password);
    let engine = STANDARD_NO_PAD;
    let encoded_credentials = engine.encode(credentials.as_bytes());
    let basic_auth = format!("Basic {}", encoded_credentials);

    // Vérifier si la page existe déjà
    let encoded_title = encode(title);
    let search_url = format!(
        "{}/wp-json/wp/v2/pages?search={}&status=draft,publish",
        wordpress_url, encoded_title
    );

    let search_response = client
        .get(search_url.as_str())
        .header("Authorization", &basic_auth)
        .send()
        .await?;

    if search_response.status().is_success() {
        let pages: Vec<Value> = search_response
            .json()
            .await
            .context("Failed to parse search response")?;

        if !pages.is_empty() {
            for page in pages {
                let found_title = page["title"]["rendered"].as_str().unwrap_or_default();
                if found_title == title {
                    return Ok(format!(
                        "Page already exists: {} - {}",
                        page["id"], found_title
                    ));
                }
            }
        }
    } else {
        let error_msg = format!("Failed to search for page: {}", search_response.status());
        anyhow::bail!(error_msg);
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
