use crate::wordpress::create_wordpress_page;

// Create WordPress page if not exists
pub async fn add_page(
    wordpress_url: &String,
    username: &String,
    password: &String,
    content: &str,
    product_id: &str,
    status: &str,
    author: i32,
    mut current_parent_id: i32,
    name: &String,
) {
    match create_wordpress_page::create_wordpress_page(
        name, // Using breadcrumb name for page title
        content,
        product_id,
        status,
        author,
        wordpress_url,
        username,
        password,
        current_parent_id,
    ).await
    {
        Ok(response) => {
            println!("Page created successfully");
            //println!("Page created successfully, raw response: {}", response);

            // Processing the JSON response to extract the ID of the created page
            match serde_json::from_str::<serde_json::Value>(
                &response,
            ) {
                Ok(json_value) => {
                    if let Some(id) = json_value
                        .get("id")
                        .and_then(|v| v.as_i64())
                    {
                        // Set this ID as the parent ID for the next pages
                        current_parent_id = id as i32;
                        println!(
                            "Updating parent_id for the next pages: {}",
                            current_parent_id
                        );
                    } else {
                        eprintln!("Failed to extract parent_id from the response");
                    }
                }
                Err(e) => {
                    eprintln!("Error while parsing the JSON response: {}", e);
                }
            }
        }
        Err(e) => eprintln!("Error creating page: {:?}", e),
    }
}
