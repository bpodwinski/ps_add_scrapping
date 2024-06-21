use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use reqwest::{Client, StatusCode};
use serde_json::json;

/// Creates a WordPress page using the provided details.
///
/// # Arguments
/// * `title` - The title of the page to create.
/// * `content` - The HTML content of the page.
/// * `product_id` - The product ID to attach as meta-data.
/// * `status` - The desired status of the page (e.g., 'draft', 'publish').
/// * `author` - The author's user ID.
/// * `wordpress_url` - The base URL of the WordPress site.
/// * `username` - The username for WordPress API authentication.
/// * `password` - The password for WordPress API authentication.
/// * `parent` - The parent ID of the page to set hierarchy.
///
/// # Returns
/// A result containing the body of the response as a string if successful, or an error if not.
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

    // Save the status before consuming the response
    let status_code = response.status();
    let response_body = response.text().await.context("Failed to read response body")?;

    match status_code {
        StatusCode::OK | StatusCode::CREATED => serde_json::from_str(&response_body)
            .context("Failed to parse JSON response"),
        StatusCode::BAD_REQUEST => {
            let error = serde_json::from_str::<serde_json::Value>(&response_body)
                .context("Failed to parse error JSON response")?;
            Err(anyhow::anyhow!(response_body))
        }
        _ => Err(anyhow::anyhow!("Failed to create page with status {}: {}", status_code, response_body))
    }
}
