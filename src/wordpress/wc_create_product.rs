use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use reqwest::{Client, StatusCode};
use serde_json::json;

/// Holds all parameters required to create a WooCommerce product.
#[derive(Debug, Clone)]
pub struct WcProductProps {
    name: String,
    status: String,
    r#type: String,
    r#virtual: bool,
    downloadable: bool,
    short_description: String,
    description: String,
    regular_price: String,
    categories: Vec<u32>,
    images: Vec<String>,
    wordpress_url: String,
    username: String,
    password: String,
}

impl WcProductProps {
    /// Constructs a new instance of `WpCreateProductParams`.
    pub fn new(
        name: String,
        status: String,
        r#type: String,
        r#virtual: bool,
        downloadable: bool,
        short_description: String,
        description: String,
        regular_price: String,
        categories: Vec<u32>,
        images: Vec<String>,
        wordpress_url: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            name,
            status,
            r#type,
            r#virtual,
            downloadable,
            short_description,
            description,
            regular_price,
            categories,
            images,
            wordpress_url,
            username,
            password,
        }
    }

    /// Creates a WooCommerce product asynchronously.
    pub async fn create_product(&self) -> Result<String> {
        let client = Client::new();
        let credentials = format!("{}:{}", self.username, self.password);
        let engine = STANDARD_NO_PAD;
        let encoded_credentials = engine.encode(credentials.as_bytes());
        let basic_auth = format!("Basic {}", encoded_credentials);

        let create_url = format!("{}/wp-json/wc/v3/products", self.wordpress_url);

        let product = json!({
            "name": self.name,
            "type": self.r#type,
            "short_description": self.short_description,
            "description": self.description,
            "regular_price": self.regular_price,
            "short_description": "test",
            "categories": self.categories.iter().map(|&id| json!({ "id": id })).collect::<Vec<_>>(),
            "images": self.images.iter().map(|url| json!({ "src": url })).collect::<Vec<_>>()
        });

        let response = client
            .post(&create_url)
            .header("Authorization", &basic_auth)
            .json(&product)
            .send()
            .await
            .context("Failed to send create product request")?;

        let status_code = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;

        match status_code {
            StatusCode::CREATED => Ok(response_body),
            StatusCode::BAD_REQUEST => Err(anyhow::anyhow!(response_body)),
            _ => Err(anyhow::anyhow!("Failed to create product with status {}: {}", status_code, response_body))
        }
    }
}
