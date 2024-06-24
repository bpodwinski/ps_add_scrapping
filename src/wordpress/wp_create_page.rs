use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use reqwest::{Client, StatusCode};
use serde_json::json;

/// Holds all parameters required to create a WordPress page.
#[derive(Debug, Clone)]
pub struct WpCreatePageParams {
    title: String,
    content: String,
    product_id: u32,
    product_url: String,
    status: String,
    author: u32,
    wordpress_url: String,
    username: String,
    password: String,
    parent: u32,
}

impl WpCreatePageParams {
    /// Constructs a new instance of `WpCreatePageParams`.
    ///
    /// # Arguments
    /// * `title` - The title of the page to create
    /// * `content` - The HTML content of the page
    /// * `product_id` - The product ID to attach as meta-data
    /// * `product_url` - The product URL to attach as meta-data
    /// * `status` - Status of the page (publish, future, draft, pending, private)
    /// * `author` - The ID for the author of the post
    /// * `wordpress_url` - The base URL of the WordPress site
    /// * `username` - The username for WordPress API authentication
    /// * `password` - The password for WordPress API authentication
    /// * `parent` - The parent ID of the page to set hierarchy
    pub fn new(title: String, content: String, product_id: u32, product_url: String, status: String, author: u32, wordpress_url: String, username: String, password: String, parent: u32) -> Self {
        Self {
            title,
            content,
            product_id,
            product_url,
            status,
            author,
            wordpress_url,
            username,
            password,
            parent,
        }
    }

    /// Creates a WordPress page asynchronously using the provided details.
    ///
    /// # Returns
    /// A `Result` which is either:
    /// - `Ok(String)` containing the body of the HTTP response upon successful creation.
    /// - `Err(anyhow::Error)` describing the error if the page creation failed.
    pub async fn wp_create_page(&self) -> Result<String> {
        let client = Client::new();
        let credentials = format!("{}:{}", self.username, self.password);
        let engine = STANDARD_NO_PAD;
        let encoded_credentials = engine.encode(credentials.as_bytes());
        let basic_auth = format!("Basic {}", encoded_credentials);

        let create_url = format!("{}/wp-json/wp/v2/pages", self.wordpress_url);
        let page = json!({
        "post_type": "page",
        "title": self.title,
        "content": self.content,
        "meta": {
            "ps_product_id": self.product_id,
            "ps_product_url": self.product_url
        },
        "status": self.status,
        "author": self.author,
        "parent": self.parent
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
            StatusCode::OK | StatusCode::CREATED => Ok(response_body),
            StatusCode::BAD_REQUEST => Err(anyhow::anyhow!(response_body)),
            _ => Err(anyhow::anyhow!("Failed to create page with status {}: {}", status_code, response_body))
        }
    }
}
