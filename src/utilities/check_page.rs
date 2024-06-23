use crate::wordpress::wp_check_page;

// Check if page already exist
pub async fn check_page(
    wordpress_url: &String,
    username: &String,
    password: &String,
    name: &String,
) -> bool {
    let check_response = wp_check_page::check_page_exists(
        name,
        wordpress_url,
        username,
        password,
    ).await;

    match check_response {
        Ok(Some(response)) => {
            println!("Page already exists, response: {}", response);
            return true;
        }
        Ok(None) => {
            println!("No existing page found, proceeding to create a new page");
        }
        Err(e) => {
            eprintln!("Error checking if page exists: {}", e);
            return true;
        }
    }
    false
}
