use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use urlencoding::encode;

pub async fn check_page_exists(
    title: &str,
    wordpress_url: &str,
    username: &str,
    password: &str,
) -> Result<Option<String>> {
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
                    let json_response = json!({
                        "status": "exists",
                        "message": "Page already exists",
                        "page_id": page["id"],
                        "title": found_title
                    });
                    let json_string = serde_json::to_string(&json_response).unwrap_or_else(|_| {
                        "{\"error\": \"Failed to serialize JSON.\"}".to_string()
                    });
                    return Ok(Some(json_string));
                }
            }
        }
    } else {
        let error_msg = format!("Failed to search for page: {}", search_response.status());
        anyhow::bail!(error_msg);
    }

    Ok(None)
}
