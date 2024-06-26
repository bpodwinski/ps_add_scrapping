use regex::Regex;

pub fn extract_id_from_url(url: &str) -> u32 {
    let re = Regex::new(r"/(\d+)-[^/]*$").unwrap();

    re.captures(url)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().parse::<u32>().ok()))
        .flatten()
        .unwrap_or(0)
}
