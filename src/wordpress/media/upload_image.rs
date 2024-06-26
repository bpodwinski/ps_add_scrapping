use anyhow::{anyhow, Result};
use reqwest::{Client, header, Response, StatusCode};
use reqwest::multipart::{Form, Part};

use crate::wordpress::main::{Auth, UploadImage};

impl UploadImage for Auth {
    async fn upload_image(
        &self,
        image_url: &str,
    ) -> Result<Response> {
        let client = Client::new();
        let headers = self.create_headers(Some("multipart/form-data"))?;


        // Download the image into memory
        let image_response = client.get(image_url).send().await?;
        if !image_response.status().is_success() {
            return Err(anyhow!("Failed to download image: Status {}", image_response.status()));
        }

        let image_content_type = image_response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let image_bytes = image_response.bytes().await?;

        // Build the URL for the media API
        let url = format!("{}/wp-json/wp/v2/media", self.base_url);
        let file_name = image_url.split('/').last().unwrap_or("default.jpg");

        let part = Part::stream(image_bytes)
            .file_name(file_name.to_string())
            .mime_str(&image_content_type)?;

        let form = Form::new().part("file", part);

        // Send the POST request
        let response = client.post(&url)
            .headers(headers)
            .multipart(form)
            .send()
            .await?;

        // Check the response status
        match response.status() {
            StatusCode::CREATED => Ok(response),
            _ => {
                let error_text = response.text().await?;
                Err(anyhow!("Failed to upload image: {}", error_text))
            }
        }
    }
}
