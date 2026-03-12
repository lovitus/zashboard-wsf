use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamRelease {
    pub tag_name: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<UpstreamAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamAsset {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    pub browser_download_url: String,
}

pub struct UiManagerState {
    pub active_version: Option<String>,
    pub base_dir: PathBuf,
    pub custom_releases_url: Option<String>,
    pub custom_download_base: Option<String>,
    pub server_port: Option<u16>,
    pub server_shutdown: Option<Arc<AtomicBool>>,
    pub storage_data: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiVersionInfo {
    pub active_version: Option<String>,
    pub downloaded_versions: Vec<DownloadedVersion>,
    pub custom_releases_url: Option<String>,
    pub custom_download_base: Option<String>,
    pub upstream_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadedVersion {
    pub tag: String,
    pub size_bytes: u64,
}

const ACTIVE_VERSION_FILE: &str = "ui_active_version.txt";
const CUSTOM_RELEASES_URL_FILE: &str = "ui_custom_releases_url.txt";
const CUSTOM_DOWNLOAD_BASE_FILE: &str = "ui_custom_download_base.txt";
const STORAGE_DATA_FILE: &str = "ui_storage_data.txt";
const DEFAULT_RELEASES_URL: &str = "https://api.github.com/repos/Zephyruso/zashboard/releases";
const DEFAULT_DOWNLOAD_BASE: &str = "https://github.com/Zephyruso/zashboard/releases/download";
const HTTP_TIMEOUT_SECS: u64 = 30;
const DOWNLOAD_TIMEOUT_SECS: u64 = 300;

fn read_trimmed_file(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path).ok().and_then(|s| {
        let s = s.trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    })
}

fn read_active_version(base_dir: &PathBuf) -> Option<String> {
    read_trimmed_file(&base_dir.join(ACTIVE_VERSION_FILE)).and_then(|s| {
        if s == "builtin" { None } else { Some(s) }
    })
}

fn write_active_version(base_dir: &PathBuf, version: Option<&str>) {
    let path = base_dir.join(ACTIVE_VERSION_FILE);
    let content = version.unwrap_or("builtin");
    if let Err(e) = std::fs::write(&path, content) {
        eprintln!("WARNING: Failed to write active version: {}", e);
    }
}

fn read_custom_url(base_dir: &PathBuf, filename: &str) -> Option<String> {
    read_trimmed_file(&base_dir.join(filename))
}

fn write_custom_url(base_dir: &PathBuf, filename: &str, url: Option<&str>) {
    let path = base_dir.join(filename);
    match url {
        Some(u) if !u.trim().is_empty() => { let _ = std::fs::write(&path, u.trim()); }
        _ => { let _ = std::fs::remove_file(&path); }
    }
}

fn read_storage_data(base_dir: &PathBuf) -> Option<String> {
    read_trimmed_file(&base_dir.join(STORAGE_DATA_FILE))
}

fn write_storage_data(base_dir: &PathBuf, data: Option<&str>) {
    let path = base_dir.join(STORAGE_DATA_FILE);
    match data {
        Some(d) if !d.is_empty() => { let _ = std::fs::write(&path, d); }
        _ => { let _ = std::fs::remove_file(&path); }
    }
}

fn dir_size(path: &PathBuf) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            } else if p.is_dir() {
                total += dir_size(&p);
            }
        }
    }
    total
}

fn build_http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("zashboard-wsf")
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))
}

pub fn init_state() -> UiManagerState {
    let base_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zashboard-wsf")
        .join("ui_versions");
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        eprintln!("WARNING: Failed to create UI versions dir: {}", e);
    }

    let active_version = read_active_version(&base_dir);
    let custom_releases_url = read_custom_url(&base_dir, CUSTOM_RELEASES_URL_FILE);
    let custom_download_base = read_custom_url(&base_dir, CUSTOM_DOWNLOAD_BASE_FILE);
    let storage_data = read_storage_data(&base_dir);

    // If a version was previously active, start the file server
    let (server_port, server_shutdown) = if let Some(ref ver) = active_version {
        let version_dir = base_dir.join(ver);
        if version_dir.join("index.html").exists() {
            match start_file_server(version_dir, storage_data.clone()) {
                Ok((port, shutdown)) => {
                    eprintln!("Resumed file server for {} on port {}", ver, port);
                    (Some(port), Some(shutdown))
                }
                Err(e) => {
                    eprintln!("WARNING: Failed to start file server for {}: {}", ver, e);
                    (None, None)
                }
            }
        } else {
            eprintln!("WARNING: Active version {} not found on disk, ignoring", ver);
            (None, None)
        }
    } else {
        (None, None)
    };

    eprintln!(
        "UI manager: base_dir={}, active={:?}, server_port={:?}",
        base_dir.display(),
        active_version,
        server_port
    );

    UiManagerState {
        active_version,
        base_dir,
        custom_releases_url,
        custom_download_base,
        server_port,
        server_shutdown,
        storage_data,
    }
}

// --- Commands ---

#[tauri::command]
pub async fn ui_fetch_releases(
    state: State<'_, Mutex<UiManagerState>>,
) -> Result<Vec<UpstreamRelease>, String> {
    let url = {
        let s = state.lock().map_err(|e| e.to_string())?;
        s.custom_releases_url
            .clone()
            .unwrap_or_else(|| DEFAULT_RELEASES_URL.to_string())
    };

    eprintln!("Fetching upstream releases from: {}", url);
    let client = build_http_client(HTTP_TIMEOUT_SECS)?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Request timed out. Check your network or try a custom URL.".to_string()
            } else if e.is_connect() {
                format!("Connection failed: {}. Check your network.", e)
            } else {
                format!("Fetch failed: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(match status.as_u16() {
            403 => "GitHub API rate limit exceeded. Try again later or use a custom URL.".to_string(),
            404 => format!("Releases not found at URL: {}", url),
            _ => format!("HTTP {}: {}", status, body.chars().take(200).collect::<String>()),
        });
    }

    let releases: Vec<UpstreamRelease> = response
        .json::<Vec<UpstreamRelease>>()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    Ok(releases)
}

#[tauri::command]
pub async fn ui_download_version(
    state: State<'_, Mutex<UiManagerState>>,
    tag: String,
) -> Result<String, String> {
    let (base_dir, download_base) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (
            s.base_dir.clone(),
            s.custom_download_base
                .clone()
                .unwrap_or_else(|| DEFAULT_DOWNLOAD_BASE.to_string()),
        )
    };

    let version_dir = base_dir.join(&tag);
    if version_dir.join("index.html").exists() {
        return Ok(format!("Version {} already downloaded", tag));
    }

    let url = format!("{}/{}/dist.zip", download_base.trim_end_matches('/'), tag);
    eprintln!("Downloading upstream UI: {}", url);

    let client = build_http_client(DOWNLOAD_TIMEOUT_SECS)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Download timed out. Check your network or try a custom download URL.".to_string()
            } else {
                format!("Download failed: {}", e)
            }
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed: HTTP {} from {}",
            response.status(),
            url
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    if bytes.len() < 100 {
        return Err("Downloaded file is too small — probably not a valid zip".to_string());
    }

    eprintln!("Downloaded {} bytes, extracting...", bytes.len());

    // Extract zip — clean up on failure
    std::fs::create_dir_all(&version_dir).map_err(|e| format!("Create dir failed: {}", e))?;

    let extract_result = extract_zip(&bytes, &version_dir);
    if let Err(e) = &extract_result {
        eprintln!("Extraction failed, cleaning up: {}", e);
        let _ = std::fs::remove_dir_all(&version_dir);
        return Err(e.clone());
    }

    // Verify extraction produced index.html
    if !version_dir.join("index.html").exists() {
        let _ = std::fs::remove_dir_all(&version_dir);
        return Err("Extraction succeeded but index.html not found — invalid archive".to_string());
    }

    eprintln!(
        "Extracted upstream UI version {} to {}",
        tag,
        version_dir.display()
    );
    Ok(format!("Downloaded version {}", tag))
}

fn extract_zip(bytes: &[u8], version_dir: &PathBuf) -> Result<(), String> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("Invalid zip file: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry {} error: {}", i, e))?;

        let name = file.name().to_string();

        // Strip common prefixes: "dist/", "zashboard-dist/", etc.
        let relative = name
            .strip_prefix("dist/")
            .or_else(|| name.strip_prefix("zashboard/"))
            .unwrap_or(&name);

        if relative.is_empty() || relative.starts_with("..") {
            continue;
        }

        let out_path = version_dir.join(relative);

        // Security: ensure the output path is within version_dir
        if !out_path.starts_with(version_dir) {
            eprintln!("WARNING: Skipping zip entry with path traversal: {}", name);
            continue;
        }

        if file.is_dir() {
            std::fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| format!("File create failed ({}): {}", relative, e))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| format!("File write failed ({}): {}", relative, e))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn ui_activate_version(
    state: State<'_, Mutex<UiManagerState>>,
    tag: String,
    storage_data: Option<String>,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    let version_dir = s.base_dir.join(&tag);
    if !version_dir.join("index.html").exists() {
        return Err(format!("Version {} not found or incomplete", tag));
    }

    // Stop existing server if running
    if let Some(ref shutdown) = s.server_shutdown {
        shutdown.store(true, Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(100));
    }

    // Persist storage data for restart recovery
    write_storage_data(&s.base_dir, storage_data.as_deref());

    // Start new file server for this version with storage data
    let (port, shutdown) = start_file_server(version_dir, storage_data.clone())?;

    s.active_version = Some(tag.clone());
    s.server_port = Some(port);
    s.server_shutdown = Some(shutdown);
    s.storage_data = storage_data;
    write_active_version(&s.base_dir, Some(&tag));

    let url = format!("http://127.0.0.1:{}", port);
    eprintln!("Activated upstream UI version {} at {}", tag, url);
    Ok(url)
}

#[tauri::command]
pub async fn ui_deactivate(
    state: State<'_, Mutex<UiManagerState>>,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    // Stop file server
    if let Some(ref shutdown) = s.server_shutdown {
        shutdown.store(true, Ordering::Relaxed);
    }

    s.active_version = None;
    s.server_port = None;
    s.server_shutdown = None;
    s.storage_data = None;
    write_active_version(&s.base_dir, None);
    write_storage_data(&s.base_dir, None);

    eprintln!("Deactivated upstream UI, switched to built-in");
    Ok("Switched to built-in UI".to_string())
}

#[tauri::command]
pub async fn ui_get_info(
    state: State<'_, Mutex<UiManagerState>>,
) -> Result<UiVersionInfo, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let mut downloaded = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&s.base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("index.html").exists() {
                let tag = entry.file_name().to_string_lossy().to_string();
                let size = dir_size(&path);
                downloaded.push(DownloadedVersion {
                    tag,
                    size_bytes: size,
                });
            }
        }
    }

    downloaded.sort_by(|a, b| b.tag.cmp(&a.tag));

    let upstream_url = s.server_port.map(|p| format!("http://127.0.0.1:{}", p));

    Ok(UiVersionInfo {
        active_version: s.active_version.clone(),
        downloaded_versions: downloaded,
        custom_releases_url: s.custom_releases_url.clone(),
        custom_download_base: s.custom_download_base.clone(),
        upstream_url,
    })
}

#[tauri::command]
pub async fn ui_set_custom_urls(
    state: State<'_, Mutex<UiManagerState>>,
    releases_url: Option<String>,
    download_base: Option<String>,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    let releases_url = releases_url.filter(|u| !u.trim().is_empty());
    let download_base = download_base.filter(|u| !u.trim().is_empty());

    write_custom_url(&s.base_dir, CUSTOM_RELEASES_URL_FILE, releases_url.as_deref());
    write_custom_url(&s.base_dir, CUSTOM_DOWNLOAD_BASE_FILE, download_base.as_deref());

    s.custom_releases_url = releases_url;
    s.custom_download_base = download_base;

    Ok("Custom URLs saved".to_string())
}

#[tauri::command]
pub async fn ui_delete_version(
    state: State<'_, Mutex<UiManagerState>>,
    tag: String,
) -> Result<String, String> {
    let base_dir = {
        let s = state.lock().map_err(|e| e.to_string())?;
        if s.active_version.as_deref() == Some(tag.as_str()) {
            return Err("Cannot delete the currently active version. Deactivate it first.".to_string());
        }
        s.base_dir.clone()
    };

    let version_dir = base_dir.join(&tag);
    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir).map_err(|e| format!("Delete failed: {}", e))?;
    }

    Ok(format!("Deleted version {}", tag))
}

// --- Local HTTP file server for upstream UI ---

fn mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "webp" => "image/webp",
        "wasm" => "application/wasm",
        "map" => "application/json",
        _ => "application/octet-stream",
    }
}

const RETURN_BUTTON_SCRIPT: &str = r#"<script>
(function(){
  var btn=document.createElement('div');
  btn.innerHTML='\u21A9 Built-in UI';
  btn.style.cssText='position:fixed;bottom:16px;right:16px;z-index:99999;background:#3b82f6;color:#fff;padding:8px 16px;border-radius:8px;cursor:pointer;font-size:13px;box-shadow:0 2px 8px rgba(0,0,0,.3);opacity:0.85;transition:opacity .2s';
  btn.onmouseenter=function(){btn.style.opacity='1'};
  btn.onmouseleave=function(){btn.style.opacity='0.85'};
  btn.onclick=function(){
    // Navigate back to built-in Tauri UI (try both URL schemes)
    window.location.href='https://tauri.localhost/';
    setTimeout(function(){window.location.href='tauri://localhost/';},500);
  };
  document.body.appendChild(btn);
})();
</script>"#;

fn start_file_server(root_dir: PathBuf, storage_data: Option<String>) -> Result<(u16, Arc<AtomicBool>), String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind HTTP server: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get server address: {}", e))?
        .port();
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to configure server: {}", e))?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_flag = shutdown.clone();

    std::thread::spawn(move || {
        eprintln!(
            "File server started on 127.0.0.1:{} serving {}",
            port,
            root_dir.display()
        );
        while !shutdown_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let root = root_dir.clone();
                    let sd = storage_data.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = handle_http_request(stream, &root, sd.as_deref()) {
                            eprintln!("HTTP handler error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    if !shutdown_flag.load(Ordering::Relaxed) {
                        eprintln!("HTTP accept error: {}", e);
                    }
                    break;
                }
            }
        }
        eprintln!("File server on port {} stopped", port);
    });

    Ok((port, shutdown))
}

fn handle_http_request(
    mut stream: std::net::TcpStream,
    root: &std::path::Path,
    storage_data: Option<&str>,
) -> Result<(), String> {
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .ok();

    // Read request headers (up to 8KB)
    let mut buf = [0u8; 8192];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Read error: {}", e))?;
    if n == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");

    // Parse "GET /path HTTP/1.1"
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return send_http_response(&mut stream, 400, "text/plain", b"Bad Request");
    }

    let raw_path = parts[1];

    // Parse path, strip query/hash
    let path = raw_path.split('?').next().unwrap_or("/");
    let path = path.split('#').next().unwrap_or("/");
    let decoded = percent_decode(path);
    let clean_path = if decoded.is_empty() || decoded == "/" {
        "index.html".to_string()
    } else {
        decoded.trim_start_matches('/').to_string()
    };

    // Security: prevent path traversal
    if clean_path.contains("..") {
        return send_http_response(&mut stream, 403, "text/plain", b"Forbidden");
    }

    let file_path = root.join(&clean_path);

    let (body, content_type) = if file_path.exists() && file_path.is_file() {
        let content =
            std::fs::read(&file_path).map_err(|e| format!("File read error: {}", e))?;
        let mime = mime_type(&clean_path);
        if mime.starts_with("text/html") {
            (inject_scripts(&content, storage_data), mime)
        } else {
            (content, mime)
        }
    } else {
        // SPA fallback: serve index.html for unknown paths
        let index = root.join("index.html");
        if index.exists() {
            let content =
                std::fs::read(&index).map_err(|e| format!("Index read error: {}", e))?;
            (inject_scripts(&content, storage_data), "text/html; charset=utf-8")
        } else {
            return send_http_response(&mut stream, 404, "text/plain", b"Not Found");
        }
    };

    send_http_response(&mut stream, 200, content_type, &body)
}

fn inject_scripts(html_bytes: &[u8], storage_b64: Option<&str>) -> Vec<u8> {
    let html = String::from_utf8_lossy(html_bytes);
    let mut result = html.to_string();

    // 1. Inject localStorage restoration script right after <head> (before SPA scripts)
    // The storage_b64 is base64(encodeURIComponent(JSON)) from the frontend
    if let Some(data) = storage_b64 {
        let storage_script = format!(
            "<script>(function(){{try{{var d=JSON.parse(decodeURIComponent(atob('{}')));for(var k in d){{if(d.hasOwnProperty(k))localStorage.setItem(k,d[k]);}}}}catch(e){{console.error('WSF storage restore:',e)}}}})()</script>",
            data
        );
        if let Some(pos) = result.find("<head>") {
            result.insert_str(pos + 6, &storage_script);
        } else if let Some(pos) = result.find("<HEAD>") {
            result.insert_str(pos + 6, &storage_script);
        } else {
            result = format!("{}{}", storage_script, result);
        }
    }

    // 2. Inject return button before </body>
    if result.contains("</body>") {
        result = result.replace("</body>", &format!("{}\n</body>", RETURN_BUTTON_SCRIPT));
    } else {
        result.push_str(RETURN_BUTTON_SCRIPT);
    }

    result.into_bytes()
}

fn send_http_response(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        status, status_text, content_type, body.len()
    );
    stream
        .write_all(header.as_bytes())
        .map_err(|e| format!("Write header: {}", e))?;
    stream
        .write_all(body)
        .map_err(|e| format!("Write body: {}", e))?;
    stream.flush().map_err(|e| format!("Flush: {}", e))?;
    Ok(())
}

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let hi = bytes.next().and_then(|c| (c as char).to_digit(16));
            let lo = bytes.next().and_then(|c| (c as char).to_digit(16));
            if let (Some(h), Some(l)) = (hi, lo) {
                result.push((h * 16 + l) as u8 as char);
            } else {
                result.push('%');
            }
        } else {
            result.push(b as char);
        }
    }
    result
}
