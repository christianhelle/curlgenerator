use openapiv3::OpenAPI;

pub async fn load_from_path(path: &str) -> Result<OpenAPI, String> {
    if is_http(path) {
        load_from_url(path).await
    } else {
        load_from_file(path)
    }
}

fn is_http(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

fn load_from_file(path: &str) -> Result<OpenAPI, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Could not open the file at {}: {}", path, e))?;

    if let Ok(doc) = serde_json::from_str::<OpenAPI>(&content) {
        return Ok(doc);
    }

    let yaml_content = convert_yaml_to_json(&content)
        .map_err(|e| format!("Failed to parse YAML from {}: {}", path, e))?;

    serde_json::from_str::<OpenAPI>(&yaml_content)
        .map_err(|e| format!("Failed to parse {}: {}", path, e))
}

async fn load_from_url(url: &str) -> Result<OpenAPI, String> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .deflate(true)
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Could not download the file at {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download {}: status {}",
            url,
            response.status()
        ));
    }

    let content = response
        .text()
        .await
        .map_err(|e| format!("Could not read response from {}: {}", url, e))?;

    if let Ok(doc) = serde_json::from_str::<OpenAPI>(&content) {
        return Ok(doc);
    }

    let yaml_content = convert_yaml_to_json(&content)
        .map_err(|e| format!("Failed to parse YAML from {}: {}", url, e))?;

    serde_json::from_str::<OpenAPI>(&yaml_content)
        .map_err(|e| format!("Failed to parse from {}: {}", url, e))
}

fn convert_yaml_to_json(yaml: &str) -> Result<String, String> {
    serde_yaml::from_str::<serde_json::Value>(yaml)
        .map(|v| serde_json::to_string(&v).unwrap_or_default())
        .map_err(|e| e.to_string())
}
