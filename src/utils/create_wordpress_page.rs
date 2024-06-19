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
    let url = format!("{}/wp-json/wp/v2/pages", wordpress_url);

    let credentials = format!("{}:{}", username, password);
    let engine = STANDARD_NO_PAD;
    let encoded_credentials = engine.encode(credentials.as_bytes());

    let basic_auth = format!("Basic {}", encoded_credentials);

    let page = json!({
        "post_type": "page",
        "title": title,
        "content": content,
        "status": status,
        "post_author": "1",
        "parent": parent_id
    });

    let response = client
        .post(&url)
        .header("Authorization", basic_auth)
        .json(&page)
        .send()
        .await?;

    // Extract and print the raw response body regardless of the status
    let body = response.text().await?;
    //println!("Raw response body: {}", body);

    // Simply return the raw body as a string
    Ok(body)
}
