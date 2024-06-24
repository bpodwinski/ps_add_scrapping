use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use htmlentity::entity::{decode, ICodedDataTrait};
use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use urlencoding::encode;

/// Represents the parameters required to check for an existing WordPress page.
#[derive(Debug, Clone)]
pub struct PageChecker {
    pub(crate) title: String,
    pub(crate) wordpress_url: String,
    pub(crate) username: String,
    pub(crate) password: String,
}

/// Checks if a WordPress page with the given title already exists.
///
/// # Arguments
/// * `title` - The title of the page to search for.
/// * `wordpress_url` - The base URL of the WordPress site.
/// * `username` - The username for WordPress API authentication.
/// * `password` - The password for WordPress API authentication.
///
/// # Returns
/// * `Ok(Some(String))` with a JSON containing page details if the page exists.
/// * `Ok(None)` if no matching page is found.
/// * `Err` with an error message if the search fails.
///
/// The returned JSON string includes:
/// - `status`: Status of the search ("exists" if found).
/// - `message`: A message indicating the result.
/// - `page_id`: The ID of the found page if it exists.
/// - `title`: The title of the found page.

impl PageChecker {
    /// Creates a new instance of `PageChecker`.
    ///
    /// # Arguments
    /// * `title` - The title of the page to search for.
    /// * `wordpress_url` - The base URL of the WordPress site.
    /// * `username` - Username for API authentication.
    /// * `password` - Password for API authentication.
    pub fn new(title: String, wordpress_url: String, username: String, password: String) -> Self {
        Self {
            title,
            wordpress_url,
            username,
            password,
        }
    }

    /// Checks if a page with the set title already exists on the WordPress site.
    ///
    /// # Returns
    /// * `Ok(Some(String))` containing a JSON string with page details if the page exists.
    /// * `Ok(None)` if no matching page is found.
    /// * `Err` with an error message if the search fails.
    ///
    /// The returned JSON string includes details such as the status, message, page ID, and title.
    pub async fn wp_check_page(&self) -> Result<Option<String>> {
        let client = Client::new();
        let credentials = format!("{}:{}", self.username, self.password);
        let engine = STANDARD_NO_PAD;
        let encoded_credentials = engine.encode(credentials.as_bytes());
        let basic_auth = format!("Basic {}", encoded_credentials);

        let encoded_title = encode(&*self.title);
        let search_url = format!(
            "{}/wp-json/wp/v2/pages?search={}&status=draft,publish",
            self.wordpress_url, encoded_title
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
                .context("Failed to parse search response as JSON")?;

            let decoded_title = decode(self.title.as_bytes()).to_string()
                .context("Failed to decode the title for comparison")?;

            if !pages.is_empty() {
                for page in pages {
                    let found_title_encoded = page["title"]["rendered"].as_str().unwrap_or_default();
                    let found_title = decode(found_title_encoded.as_bytes()).to_string()?;

                    if found_title == decoded_title {
                        let json_response = json!({
                            "status": "exists",
                            "message": "Page already exists",
                            "page_id": page["id"],
                            "title": found_title
                        });
                        let json_string = serde_json::to_string(&json_response)
                            .with_context(|| "Failed to serialize response JSON")?;
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
}
