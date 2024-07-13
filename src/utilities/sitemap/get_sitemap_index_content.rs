use std::sync::Arc;

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::get_configuration::get_configuration_value;

#[derive(Serialize)]
struct RequestPayload<'a> {
    cmd: &'a str,
    url: &'a str,
    user_agent: &'a str,
}

#[derive(Deserialize)]
struct ResponsePayload {
    solution: Solution,
}

#[derive(Deserialize)]
struct Solution {
    response: String,
}

/// Fetches the robots.txt file from the given URL, extracts the sitemap URL, and retrieves the content of the sitemap.
///
/// This function performs the following steps:
///
/// 1. Sends a GET request to the specified URL to retrieve the robots.txt file.
/// 2. Searches for a line starting with "Sitemap:" and extracts the URL.
/// 3. Uses Flaresolverr to send a GET request to the extracted sitemap URL and retrieves the content.
///
/// # Errors
///
/// This function will return an error if any of the following operations fail:
///
/// - Sending the GET request to the specified URL.
/// - Parsing the response body as text.
/// - Finding a valid sitemap URL in the response body.
/// - Sending the GET request to the sitemap URL.
/// - Reading the content of the sitemap.
///
/// # Returns
///
/// If successful, returns the content of the sitemap as a string.
pub async fn get_sitemap_index_content(conn: Arc<Mutex<Connection>>, robots_url: &str) -> Result<String> {
    let flaresolverr_url = get_configuration_value(conn.clone(), "flaresolverr_url").await?;
    let user_agent = get_configuration_value(conn.clone(), "user_agent").await?;

    // Create an HTTP client
    let client = Client::new();

    // Prepare the request payload for robots.txt
    let robots_payload = RequestPayload {
        cmd: "request.get",
        url: robots_url,
        user_agent: &user_agent,
    };

    // Send the request via Flaresolverr for robots.txt
    let robots_response = client
        .post(&flaresolverr_url)
        .json(&robots_payload)
        .send()
        .await
        .context("Failed to send request to Flaresolverr")?;

    let raw_robots_response = robots_response.text().await.context("Failed to read raw response body")?;

    let robots_response_payload: ResponsePayload = serde_json::from_str(&raw_robots_response)
        .context("Failed to parse Flaresolverr response as JSON")?;

    let robots_body = robots_response_payload.solution.response;

    let pre_content = extract_pre_content(&robots_body).context("Failed to extract content from <pre> tags")?;

    let re = Regex::new(r"(?im)^Sitemap:\s*(?P<url>https?://\S+)$")
        .context("Failed to compile regex")?;

    let sitemap_url = if let Some(captures) = re.captures(&pre_content) {
        if let Some(sitemap_url) = captures.name("url") {
            sitemap_url.as_str().to_string()
        } else {
            return Err(anyhow::anyhow!("Failed to find sitemap URL in robots.txt"));
        }
    } else {
        return Err(anyhow::anyhow!("Failed to find sitemap URL in robots.txt"));
    };

    // Prepare the request payload for the sitemap
    let sitemap_payload = RequestPayload {
        cmd: "request.get",
        url: &sitemap_url,
        user_agent: &user_agent,
    };

    // Send the request via Flaresolverr for the sitemap
    let sitemap_response = client
        .post(flaresolverr_url)
        .json(&sitemap_payload)
        .send()
        .await
        .context("Failed to send request to Flaresolverr")?;

    let raw_sitemap_response = sitemap_response.text().await.context("Failed to read raw response body")?;

    let sitemap_response_payload: ResponsePayload = serde_json::from_str(&raw_sitemap_response)
        .context("Failed to parse Flaresolverr response as JSON")?;

    let sitemap_body = sitemap_response_payload.solution.response;

    // Extract content inside <sitemapindex> tags
    let cleaned_sitemap_content = extract_sitemap_content(&sitemap_body)
        .context("Failed to extract content from <sitemapindex> tags")?;

    Ok(cleaned_sitemap_content)
}

/// Extracts content inside <pre> tags from the given HTML string.
fn extract_pre_content(html: &str) -> Option<String> {
    let pre_re = Regex::new(r"(?s)<pre.*?>(.*?)</pre>").ok()?;
    pre_re.captures(html).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Extracts content inside <sitemapindex> tags from the given HTML string.
fn extract_sitemap_content(html: &str) -> Option<String> {
    let sitemap_re = Regex::new(r"(?s)<sitemapindex[^>]*>(.*?)</sitemapindex>").ok()?;
    sitemap_re.captures(html).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}