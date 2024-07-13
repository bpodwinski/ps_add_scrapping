use anyhow::Result;
use chrono::{Duration, Utc};
use colored::Colorize;
use rusqlite::Connection;

use crate::utilities::database::insert_xml_into_sql::insert_xml_into_sql;
use crate::utilities::sitemap::get_sitemap_index_content::get_sitemap_index_content;
use crate::utilities::sitemap::get_sitemap_urls_content::get_sitemap_urls_content;

pub async fn sitemap_update(
    conn: &Connection,
    days: i64,
) -> Result<()> {

    // Check the last sitemap update date
    let skip_sitemap = if let Ok(Some(last_insert_date)) = get_last_xml_insert_date(&conn) {
        let last_insert_date = chrono::DateTime::parse_from_rfc3339(&last_insert_date)?;
        let now = Utc::now();
        let duration = now.signed_duration_since(last_insert_date);

        if duration < Duration::days(days) {
            println!("{}", format!("Last sitemap update was less than {} days ago. Skipping update", days).yellow());
            true
        } else {
            false
        }
    } else {
        false
    };

    if !skip_sitemap {
        // Extract content for sitemap index
        let sitemap_index_content = match get_sitemap_index_content("https://addons.prestashop.com/robots.txt").await { // TODO: use config variable for robots_url
            Ok(content) => content,
            Err(e) => {
                eprintln!("{}", format!("Failed to extract sitemap index URL and content: {:?}", e).red());
                return Err(e.into());
            }
        };

        // Extract content for sitemap url
        let sitemap_urls_content = match get_sitemap_urls_content(&sitemap_index_content, "fr").await { // TODO: use config variable for sitemap_lang
            Ok(content) => {
                content
            }
            Err(e) => {
                eprintln!("{}", format!("Failed to fetch sitemap URL and content: {:?}", e).red());
                return Err(e.into());
            }
        };

        // Insert sitemap urls into database
        match insert_xml_into_sql(&conn, &sitemap_urls_content) {
            Ok(_) => println!("{}", "Successfully inserted data into the database.".green()),
            Err(e) => {
                eprintln!("{}", format!("Failed to insert sitemap URLs into database: {:?}", e).red());
                return Err(e.into());
            }
        }
    } else {
        println!("{}", "Skipping the XML insertion process.".yellow());
    }

    Ok(())
}

fn get_last_xml_insert_date(conn: &Connection) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM configuration WHERE key = 'last_sitemap_insert_date'")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let date: String = row.get(0)?;
        Ok(Some(date))
    } else {
        Ok(None)
    }
}