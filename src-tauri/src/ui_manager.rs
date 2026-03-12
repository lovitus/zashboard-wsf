use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

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
}

#[derive(Debug, Clone, Serialize)]
pub struct UiVersionInfo {
    pub active_version: Option<String>,
    pub downloaded_versions: Vec<DownloadedVersion>,
    pub custom_releases_url: Option<String>,
    pub custom_download_base: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadedVersion {
    pub tag: String,
    pub size_bytes: u64,
}

const ACTIVE_VERSION_FILE: &str = "ui_active_version.txt";
const CUSTOM_RELEASES_URL_FILE: &str = "ui_custom_releases_url.txt";
const CUSTOM_DOWNLOAD_BASE_FILE: &str = "ui_custom_download_base.txt";
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

    eprintln!(
        "UI manager: base_dir={}, active={:?}, custom_url={:?}",
        base_dir.display(),
        active_version,
        custom_releases_url
    );

    UiManagerState {
        active_version,
        base_dir,
        custom_releases_url,
        custom_download_base,
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
    app: AppHandle,
    state: State<'_, Mutex<UiManagerState>>,
    tag: String,
) -> Result<String, String> {
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        let version_dir = s.base_dir.join(&tag);
        if !version_dir.join("index.html").exists() {
            return Err(format!("Version {} not found or incomplete", tag));
        }
        s.active_version = Some(tag.clone());
        write_active_version(&s.base_dir, Some(&tag));
    }

    // Desktop: open upstream UI in a new window
    #[cfg(desktop)]
    {
        open_upstream_window(&app)?;
    }

    Ok(format!("Activated version {}", tag))
}

#[cfg(desktop)]
fn open_upstream_window(app: &AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    // Close existing upstream window if open
    if let Some(existing) = app.get_webview_window("upstream-ui") {
        let _ = existing.close();
        // Small delay to let the window close
        std::thread::sleep(Duration::from_millis(100));
    }

    let url = "zui://localhost/".parse::<url::Url>().map_err(|e| format!("URL parse error: {}", e))?;

    WebviewWindowBuilder::new(app, "upstream-ui", WebviewUrl::External(url))
        .title("Zashboard (Upstream)")
        .inner_size(1200.0, 800.0)
        .min_inner_size(400.0, 600.0)
        .build()
        .map_err(|e| format!("Failed to open upstream UI window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn ui_open_upstream(app: AppHandle) -> Result<String, String> {
    #[cfg(desktop)]
    {
        open_upstream_window(&app)?;
        Ok("Upstream UI window opened".to_string())
    }
    #[cfg(not(desktop))]
    {
        let _ = app;
        Err("Opening upstream UI window is only supported on desktop".to_string())
    }
}

#[tauri::command]
pub async fn ui_deactivate(
    app: AppHandle,
    state: State<'_, Mutex<UiManagerState>>,
) -> Result<String, String> {
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.active_version = None;
        write_active_version(&s.base_dir, None);
    }

    // Close upstream UI window if open
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("upstream-ui") {
        let _ = window.close();
    }

    let _ = app; // suppress unused warning on mobile

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

    Ok(UiVersionInfo {
        active_version: s.active_version.clone(),
        downloaded_versions: downloaded,
        custom_releases_url: s.custom_releases_url.clone(),
        custom_download_base: s.custom_download_base.clone(),
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

// --- Protocol handler ---

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
    try{
      if(window.__TAURI_INTERNALS__){
        window.__TAURI_INTERNALS__.invoke('ui_deactivate').catch(function(){});
      }
    }catch(e){}
    setTimeout(function(){try{window.close()}catch(e){}},200);
  };
  document.body.appendChild(btn);
})();
</script>"#;

pub fn handle_zui_protocol<R: tauri::Runtime>(
    ctx: &tauri::UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    let app = ctx.app_handle();
    let state = app.state::<Mutex<UiManagerState>>();
    let s = match state.lock() {
        Ok(s) => s,
        Err(_) => {
            return tauri::http::Response::builder()
                .status(500)
                .header("Content-Type", "text/plain")
                .body(b"Internal error: state lock failed".to_vec())
                .unwrap();
        }
    };

    let active = match &s.active_version {
        Some(v) => v.clone(),
        None => {
            return tauri::http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(b"<html><body><h2>No upstream UI active</h2><p>Select a version in the main window settings.</p><script>setTimeout(function(){try{window.close()}catch(e){}},2000);</script></body></html>".to_vec())
                .unwrap();
        }
    };

    let version_dir = s.base_dir.join(&active);
    drop(s);

    if !version_dir.exists() {
        return tauri::http::Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(format!("<html><body><h2>Version {} files not found</h2><p>The version directory may have been deleted.</p></body></html>", active).into_bytes())
            .unwrap();
    }

    // Parse path from URI
    let uri = request.uri().to_string();
    let path = if let Some(rest) = uri.strip_prefix("zui://localhost") {
        if rest.is_empty() { "/" } else { rest }
    } else if let Some(rest) = uri.strip_prefix("zui://") {
        if rest.is_empty() { "/" } else { rest }
    } else {
        "/"
    };

    // Strip query string and hash
    let path = path.split('?').next().unwrap_or("/");
    let path = path.split('#').next().unwrap_or("/");
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    // Try to read the file
    let file_path = version_dir.join(path);
    let (body, content_type) = if file_path.exists() && file_path.is_file() {
        let mut content = Vec::new();
        if let Ok(mut f) = std::fs::File::open(&file_path) {
            let _ = f.read_to_end(&mut content);
        }
        let mime = mime_type(path);

        // Inject return button into HTML
        if mime.starts_with("text/html") {
            let html = String::from_utf8_lossy(&content);
            let modified = if html.contains("</body>") {
                html.replace("</body>", &format!("{}</body>", RETURN_BUTTON_SCRIPT))
            } else {
                format!("{}{}", html, RETURN_BUTTON_SCRIPT)
            };
            (modified.into_bytes(), mime)
        } else {
            (content, mime)
        }
    } else {
        // SPA fallback: serve index.html for non-file paths
        let index_path = version_dir.join("index.html");
        if let Ok(mut f) = std::fs::File::open(&index_path) {
            let mut content = Vec::new();
            let _ = f.read_to_end(&mut content);
            let html = String::from_utf8_lossy(&content);
            let modified = if html.contains("</body>") {
                html.replace("</body>", &format!("{}</body>", RETURN_BUTTON_SCRIPT))
            } else {
                format!("{}{}", html, RETURN_BUTTON_SCRIPT)
            };
            (modified.into_bytes(), "text/html; charset=utf-8")
        } else {
            return tauri::http::Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(format!("File not found: {}", path).into_bytes())
                .unwrap();
        }
    };

    tauri::http::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        .unwrap()
}
