use std::sync::Arc;

use anyhow::Result;
use chrono::{Duration, Utc};
use colored::Colorize;
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::config::get_configuration::get_configuration_value;
use crate::utilities::database::insert_sitemap_into_sql::insert_sitemap_into_sql;
use crate::utilities::sitemap::get_sitemap_index_content::get_sitemap_index_content;
use crate::utilities::sitemap::get_sitemap_urls_content::get_sitemap_urls_content;

pub async fn sitemap_update(
    db: &Arc<Mutex<Connection>>,
    sitemap_frequency_update: i64,
) -> Result<()> {
    let robots_url = get_configuration_value(db, "robots_url").await?;
    let sitemap_lang = get_configuration_value(db, "sitemap_lang").await?;

    // Check the last sitemap update date
    let skip_sitemap = if let Ok(Some(last_insert_date)) = get_last_xml_insert_date(db).await {
        let last_insert_date = chrono::DateTime::parse_from_rfc3339(&last_insert_date)?;
        let now = Utc::now();
        let duration = now.signed_duration_since(last_insert_date);

        if duration < Duration::days(sitemap_frequency_update) {
            println!("{}", format!("Last sitemap update was less than {} days ago", sitemap_frequency_update).yellow());
            true
        } else {
            false
        }
    } else {
        false
    };

    if !skip_sitemap {
        // Extract content for sitemap index
        let sitemap_index_content = match get_sitemap_index_content(db, &robots_url).await {
            Ok(content) => content,
            Err(e) => {
                eprintln!("{}", format!("Failed to extract sitemap index data: {:?}", e).red());
                return Err(e.into());
            }
        };

        // Extract content for sitemap url
        let sitemap_urls_content = match get_sitemap_urls_content(db, &sitemap_index_content, &sitemap_lang).await {
            Ok(content) => {
                content
            }
            Err(e) => {
                eprintln!("{}", format!("Failed to fetch sitemap url data: {:?}", e).red());
                return Err(e.into());
            }
        };

        // Insert sitemap urls into database
        match insert_sitemap_into_sql(&db, &sitemap_urls_content).await {
            Ok(_) => println!("{}", "Added sitemap data successfully into database".green()),
            Err(e) => {
                eprintln!("{}", format!("Failed to added sitemap data into database: {:?}", e).red());
                return Err(e.into());
            }
        }
    } else {
        println!("{}", "Skipping update sitemap".yellow());
    }

    Ok(())
}

async fn get_last_xml_insert_date(
    db: &Arc<Mutex<Connection>>
) -> Result<Option<String>> {
    let db = db.lock().await;
    let mut stmt = db.prepare("SELECT value FROM configuration WHERE key = 'last_sitemap_insert_date'")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let date: String = row.get(0)?;
        Ok(Some(date))
    } else {
        Ok(None)
    }
}