use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;

pub async fn insert_sitemap_into_sql(
    db: &Arc<Mutex<Connection>>,
    xml_content: &str,
) -> Result<()> {
    let db = db.lock().await;

    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut url = String::new();
    let mut last_mod = String::new();
    let mut change_freq = String::new();

    let base_url = "https://addons.prestashop.com";
    let content_url = "/content/";
    let product_url = Regex::new(r"^\d+-").unwrap();

    let mut skip = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"loc" => {
                    url = reader.read_text(e.name()).context("Failed to read loc text")?.parse()?;

                    // Skip non-product URLs
                    if url.contains(base_url) && url.contains(content_url) {
                        eprintln!("Skipping URL: {}", url);
                        skip = true;
                    } else if url.starts_with(base_url) {
                        // Remove the base URL and split the remaining part
                        let path = &url[base_url.len()..];
                        let parts: Vec<&str> = path.split('/').collect();

                        // Check if the path follows the pattern of a product URL
                        if parts.len() >= 4 {
                            let id_and_name = parts[3];
                            // Check if it follows the pattern of product URL
                            if product_url.is_match(id_and_name) && id_and_name.ends_with(".html") {
                                skip = false;
                            } else {
                                eprintln!("Skipping URL: {}", url);
                                skip = true;
                            }
                        } else {
                            eprintln!("Skipping URL: {}", url);
                            skip = true;
                        }
                    } else {
                        eprintln!("Skipping URL: {}", url);
                        skip = true;
                    }
                }
                b"lastmod" => {
                    last_mod = reader.read_text(e.name()).context("Failed to read lastmod text")?.parse()?;

                    // Verify the date format
                    if last_mod.parse::<DateTime<FixedOffset>>().is_err() {
                        eprintln!("Invalid date format: {}", last_mod);
                        skip = true;
                    }
                }
                b"changefreq" => {
                    change_freq = reader.read_text(e.name()).context("Failed to read changefreq text")?.parse()?;
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"url" {

                    // Skip the insertion if marked to skip
                    if skip {
                        continue;
                    }

                    // Insert or update the data
                    db.execute(
                        "INSERT INTO urls (url, last_mod, change_freq) VALUES (?1, ?2, ?3)
                        ON CONFLICT(url) DO UPDATE SET last_mod = excluded.last_mod, change_freq = excluded.change_freq",
                        params![url, last_mod, change_freq],
                    ).context("Failed to insert or update data in the database")?;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("Error at position {}: {:?}", reader.buffer_position(), e)),
            _ => (),
        }
        buf.clear();
    }

    // Insert the current date and time into the configuration table
    let current_date = Utc::now().to_rfc3339();
    db.execute(
        "INSERT INTO configuration (key, value) VALUES ('last_sitemap_insert_date', ?1)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![current_date],
    ).context("Failed to insert or update last_xml_insert_date in the configuration table")?;

    Ok(())
}