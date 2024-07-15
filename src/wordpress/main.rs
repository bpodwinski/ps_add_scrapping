use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::wordpress::woocommerce::create_product::ProductCreationResult;
use crate::wordpress::woocommerce::find_category::CategoryInfo;
use crate::wordpress::woocommerce::find_product::ProductInfo;

pub struct Auth {
    pub base_url: String,
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
    /// Creates a product in WordPress WooCommerce using the provided details.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the product.
    /// * `status` - The status of the product (e.g., "draft", "publish").
    /// * `type` - The type of the product (e.g., "simple", "variable").
    /// * `virtual` - Whether the product is virtual.
    /// * `downloadable` - Whether the product is downloadable.
    /// * `short_description` - A short description of the product.
    /// * `description` - A detailed description of the product.
    /// * `regular_price` - The regular price of the product.
    /// * `categories` - A vector of category IDs to which the product belongs.
    /// * `images` - A vector of image URLs for the product.
    /// * `ps_product_id` - The PrestaShop product ID.
    /// * `ps_product_url` - The PrestaShop product URL.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response from the WordPress API as a `Value` on success,
    /// or an error on failure.
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
    ) -> Result<ProductCreationResult>;
}

pub trait FindProductByCustomField {
    async fn find_product_by_custom_field(
        &self,
        name: &str,
        status: &str,
    ) -> Result<ProductInfo>;
}

// pub trait CreatePage {
//     async fn create_page(&self, title: &str, content: &str, product_id: u32, product_url: &str, status: &str, author: u32, parent: u32) -> Result<String>;
// }

// pub trait FindPage {
//     async fn find_page(&self, title: &str) -> Result<Option<String>>;
// }

pub trait FindCategoryByCustomField {
    /// Implementation of the `FindCategoryByCustomField` trait for the `Auth` struct.
    ///
    /// This function sends a request to the WordPress WooCommerce API to search for a product category
    /// with the provided PrestaShop addons category ID. It constructs the necessary API URL, makes the HTTP request,
    /// and handles the response to determine if the category exists.
    ///
    /// # Arguments
    ///
    /// * `custom_field` - The PrestaShop addons category ID to search for.
    ///
    /// # Returns
    ///
    /// If successful, returns a `CategoryInfo` struct containing the status, message, and category details
    /// if the category is found.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the following operations fail:
    ///
    /// - Constructing the HTTP headers.
    /// - Sending the HTTP request.
    /// - Parsing the response body as JSON.
    async fn find_category_by_custom_field(&self, custom_field: u32) -> Result<CategoryInfo>;
}

pub trait CreateCategory {
    /// Implementation of the `CreateCategory` trait for the `Auth` struct.
    ///
    /// This function sends a request to the WordPress WooCommerce API to create a new product category
    /// with the provided details. It constructs the necessary JSON payload, makes the HTTP request,
    /// and handles the response.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the category to be created.
    /// * `parent` - The ID of the parent category.
    /// * `ps_addons_cat_id` - The ID of the PrestaShop category.
    ///
    /// # Returns
    ///
    /// If successful, returns the response from the WordPress API as a `Value` on success,
    /// or an error on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the following operations fail:
    ///
    /// - Constructing the HTTP headers.
    /// - Sending the HTTP request.
    /// - Reading the response body.
    /// - Parsing the response body as JSON.
    async fn create_category(&self, name: String, parent: u32, ps_addons_cat_id: u32) -> Result<Value>;
}

// pub trait UploadImage {
//     async fn upload_image(&self, image_url: &str) -> Result<Response>;
// }