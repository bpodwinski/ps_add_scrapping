use std::sync::Arc;

use crate::extract_data::ScrapedData;
use crate::MediaResponse;
use crate::wordpress::main::{Auth, UploadImage};

pub(crate) async fn process_images(
    wordpress_url: &str,
    username: &str,
    password: &str,
    extract_data:
    &ScrapedData,
) -> String {
    let wp = Arc::new(Auth::new(wordpress_url.to_string(), username.to_string(), password.to_string()));
    let mut formatted_strings = Vec::new();

    for image_url in &*extract_data.image_urls {
        println!("Uploading image from URL: {}", image_url);

        match wp.upload_image(image_url).await {
            Ok(response) => {
                println!("Image uploaded successfully: {}", image_url);
                if let Ok(media_response) = response.json::<MediaResponse>().await {
                    let formatted_string = format!("[fusion_image linktarget=\"_self\" image=\"{}\" image_id=\"{}|full\" /]", media_response.source_url, media_response.id);
                    formatted_strings.push(formatted_string);
                } else {
                    eprintln!("Failed to parse response as JSON: {}", image_url);
                }
            }
            Err(e) => {
                eprintln!("Failed to upload image: {}. Error: {}", image_url, e);
                continue;
            }
        }
    };

    let mut content = String::from(r#"[fusion_images order_by="desc" picture_size="auto" hover_type="none" autoplay="no" flex_align_items="center" columns="3" column_spacing="5" show_nav="yes" mouse_scroll="no" border="no" lightbox="yes" caption_style="off" caption_title_tag="2" caption_align_medium="none" caption_align_small="none" caption_align="none" hide_on_mobile="small-visibility,medium-visibility,large-visibility"]"#);
    let images_tags = formatted_strings.join("");
    content.push_str(&images_tags);
    content.push_str("\n[/fusion_images]");
    images_tags
}
