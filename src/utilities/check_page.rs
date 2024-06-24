use crate::wordpress::wp_check_page::PageChecker;

// Check if page already exist
pub async fn check_page(
    wordpress_url: &str,
    username: &str,
    password: &str,
    name: &str,
) -> bool {
    let title = name.to_string();
    let wordpress_url = wordpress_url.to_string();
    let username = username.to_string();
    let password = password.to_string();

    let page_checker = PageChecker::new(title, wordpress_url, username, password);

    match page_checker.wp_check_page().await {
        Ok(Some(json_result)) => {
            println!("Page found: {}", json_result);
            return true;
        }
        Ok(None) => {
            println!("No matching page found.");
            return false;
        }
        Err(e) => {
            println!("Error occurred: {}", e);
            return true;
        }
    }
}
