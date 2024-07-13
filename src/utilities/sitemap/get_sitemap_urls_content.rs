use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

/// Fetches the sitemap URLs based on the given language, retrieves the content via Flaresolverr, and returns the cleaned XML response.
///
/// # Arguments
///
/// * `sitemap_index_content` - The content of the sitemap index as a &str.
/// * `sitemap_lang` - The language code to filter the sitemap URLs.
///
/// # Errors
///
/// Returns an error if any of the following operations fail:
///
/// - Parsing the sitemap content.
/// - Sending the GET request to the sitemap URL.
/// - Reading the response body.
///
/// # Returns
///
/// If successful, returns the cleaned XML content as a `String`.
pub async fn get_sitemap_urls_content(sitemap_index_content: &str, sitemap_lang: &str) -> Result<String> {
    let flaresolverr_url = "http://flare.solpheo.com/v1"; // TODO: use config variable for flaresolverr_url

    // Create an HTTP client
    let client = Client::new();

    let mut reader = Reader::from_str(sitemap_index_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_url_tag = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"sitemap" => {
                in_url_tag = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"sitemap" => {
                in_url_tag = false;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"loc" && in_url_tag => {
                let url = reader.read_text(e.name()).context("Failed to read loc text")?;
                if url.contains(&format!("sitemap_{}", sitemap_lang)) {
                    // Prepare the request payload for the sitemap URL
                    let sitemap_payload = RequestPayload {
                        cmd: "request.get",
                        url: &url,
                        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36", // TODO: use config variable for user_agent
                    };

                    // Send the request via Flaresolverr for the sitemap URL
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

                    // Extract and clean the XML content
                    let cleaned_sitemap_content = extract_xml_content(&sitemap_body)
                        .context("Failed to extract XML content from response")?;

                    // Return the cleaned XML content
                    return Ok(cleaned_sitemap_content);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("Error at position {}: {:?}", reader.buffer_position(), e)),
            _ => (),
        }
        buf.clear();
    }

    Err(anyhow::anyhow!("No matching sitemap URL found"))
}

/// Extracts the content inside <urlset> tags from the given HTML string.
fn extract_xml_content(html: &str) -> Option<String> {
    let urlset_re = Regex::new(r"(?s)<urlset[^>]*>(.*?)</urlset>").ok()?;
    urlset_re.captures(html).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}