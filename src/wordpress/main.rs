use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::Response;
use serde_json::Value;

use crate::wordpress::woocommerce::find_category_custom_ps_addons_cat_id::CategoryInfo;

pub struct Auth {
    pub(crate) base_url: String,
    username: String,
    password: String,
}

impl Auth {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Auth {
            base_url,
            username,
            password,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Creates headers for HTTP requests, defaulting to application/json.
    /// Allows overriding the content type for other uses, like multipart/form-data.
    pub fn create_headers(&self, content_type: Option<&str>) -> Result<HeaderMap> {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded_credentials = STANDARD_NO_PAD.encode(credentials.as_bytes());
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", encoded_credentials))?);

        let content_type = content_type.unwrap_or("application/json");
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type)?);

        Ok(headers)
    }
}

pub trait CreateProduct {
    async fn create_product(
        &self,
        name: String,
        status: String,
        r#type: String,
        r#virtual: bool,
        downloadable: bool,
        short_description: String,
        description: String,
        regular_price: String,
        categories: Vec<u32>,
        images: &Vec<String>,
        ps_product_id: u32,
        ps_product_url: String,
    ) -> Result<String>;
}

pub trait CreatePage {
    async fn create_page(&self, title: &str, content: &str, product_id: u32, product_url: &str, status: &str, author: u32, parent: u32) -> Result<String>;
}

pub trait FindPage {
    async fn find_page(&self, title: &str) -> Result<Option<String>>;
}

pub trait FindCategoryCustomPsAddonsCatId {
    async fn find_category_custom_ps_addons_cat_id(&self, ps_addons_cat_id: u32) -> Result<CategoryInfo>;
}

pub trait CreateCategory {
    async fn create_category(&self, name: String, parent: u32, ps_addons_cat_id: u32) -> Result<Value>;
}

pub trait UploadImage {
    async fn upload_image(&self, image_url: &str) -> Result<Response>;
}
