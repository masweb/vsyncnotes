use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde_json::Value;

pub struct WebDavClient {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl WebDavClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Result<Self> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    fn url(&self, path: &str) -> String {
        if path.is_empty() {
            self.base_url.clone()
        } else {
            format!("{}/{}", self.base_url, path.trim_start_matches('/'))
        }
    }

    pub async fn ensure_dir(&self, subdir: &str) -> Result<()> {
        let url = self.url(&format!("{}/", subdir));
        let resp = self
            .client
            .request(reqwest::Method::from_bytes(b"MKCOL").expect("valid HTTP method"), &url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        let status = resp.status().as_u16();
        // 201 Created or 405 Already Exists — both fine
        if status != 201 && status != 405 {
            return Err(anyhow!("MKCOL {} failed: {}", subdir, status));
        }
        Ok(())
    }

    /// Returns .json filenames present in a remote subdirectory.
    pub async fn list_json_files(&self, subdir: &str) -> Result<Vec<String>> {
        let url = self.url(&format!("{}/", subdir));
        let body = r#"<?xml version="1.0" encoding="utf-8"?><D:propfind xmlns:D="DAV:"><D:prop><D:resourcetype/></D:prop></D:propfind>"#;

        let resp = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").expect("valid HTTP method"), &url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .basic_auth(&self.username, Some(&self.password))
            .body(body)
            .send()
            .await?;

        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        if status.as_u16() != 207 && !status.is_success() {
            return Err(anyhow!("PROPFIND {} failed: {}", subdir, status));
        }

        let text = resp.text().await?;
        Ok(extract_json_filenames(&text))
    }

    pub async fn read_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get(self.url(path))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("GET {} failed: {}", path, resp.status()));
        }
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn write_bytes(&self, path: &str, data: Vec<u8>) -> Result<()> {
        let resp = self
            .client
            .put(self.url(path))
            .basic_auth(&self.username, Some(&self.password))
            .body(data)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("PUT {} failed: {}", path, resp.status()));
        }
        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let resp = self
            .client
            .delete(self.url(path))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        // 200 OK, 204 No Content, or 404 Not Found (already gone) are all fine
        let status = resp.status().as_u16();
        if status != 200 && status != 204 && status != 404 {
            return Err(anyhow!("DELETE {} failed: {}", path, status));
        }
        Ok(())
    }

    /// Verifies connectivity and credentials by doing a PROPFIND Depth:0 on the base URL.
    pub async fn test_connection(&self) -> Result<()> {
        let body = r#"<?xml version="1.0" encoding="utf-8"?><D:propfind xmlns:D="DAV:"><D:prop><D:resourcetype/></D:prop></D:propfind>"#;
        let resp = self
            .client
            .request(
                reqwest::Method::from_bytes(b"PROPFIND").expect("valid HTTP method"),
                &self.base_url,
            )
            .header("Depth", "0")
            .header("Content-Type", "application/xml; charset=utf-8")
            .basic_auth(&self.username, Some(&self.password))
            .body(body)
            .send()
            .await
            .map_err(|e| anyhow!("No se pudo conectar al servidor: {e}"))?;

        match resp.status().as_u16() {
            207 | 200 => Ok(()),
            401 | 403 => Err(anyhow!("Credenciales incorrectas")),
            404 => Err(anyhow!("URL no encontrada")),
            s => Err(anyhow!("El servidor respondió con código {s}")),
        }
    }

    pub async fn file_exists(&self, path: &str) -> bool {
        self.client
            .head(self.url(path))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn read_updated_at(&self, path: &str) -> Result<DateTime<Utc>> {
        let bytes = self.read_bytes(path).await?;
        let v: Value = serde_json::from_slice(&bytes)?;
        let ts = v["updated_at"]
            .as_str()
            .ok_or_else(|| anyhow!("missing updated_at in {}", path))?;
        Ok(ts.parse::<DateTime<Utc>>()?)
    }
}

/// Extracts .json filenames from a WebDAV PROPFIND XML response.
/// Uses simple string matching — avoids an XML parser dependency.
fn extract_json_filenames(xml: &str) -> Vec<String> {
    let mut files = Vec::new();

    // WebDAV href tags can appear as <D:href>, <d:href>, or <href>
    for open_tag in &["<D:href>", "<d:href>", "<href>"] {
        let close_tag = open_tag.replace('<', "</");
        let mut pos = 0;
        while let Some(rel) = xml[pos..].find(open_tag) {
            let content_start = pos + rel + open_tag.len();
            if let Some(end_rel) = xml[content_start..].find(close_tag.as_str()) {
                let href = xml[content_start..content_start + end_rel].trim();
                if let Some(fname) = href.split('/').filter(|s| !s.is_empty()).last() {
                    let fname = percent_decode(fname);
                    if fname.ends_with(".json") && !files.contains(&fname) {
                        files.push(fname);
                    }
                }
                pos = content_start + end_rel + close_tag.len();
            } else {
                break;
            }
        }
        if !files.is_empty() {
            break; // found with this tag style, stop trying others
        }
    }

    files
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}
