use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
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

fn build_state(base_dir: PathBuf) -> UiManagerState {
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        eprintln!("WARNING: Failed to create UI versions dir: {}", e);
    }

    let active_version = read_active_version(&base_dir);
    let custom_releases_url = read_custom_url(&base_dir, CUSTOM_RELEASES_URL_FILE);
    let custom_download_base = read_custom_url(&base_dir, CUSTOM_DOWNLOAD_BASE_FILE);

    // Validate that active version still exists on disk
    if let Some(ref ver) = active_version {
        let version_dir = base_dir.join(ver);
        if !version_dir.join("index.html").exists() {
            eprintln!("WARNING: Active version {} not found on disk, resetting to built-in", ver);
            write_active_version(&base_dir, None);
            return UiManagerState {
                active_version: None,
                base_dir,
                custom_releases_url,
                custom_download_base,
            };
        }
    }

    eprintln!(
        "UI manager: base_dir={}, active={:?}",
        base_dir.display(),
        active_version,
    );

    UiManagerState {
        active_version,
        base_dir,
        custom_releases_url,
        custom_download_base,
    }
}

pub fn init_state() -> UiManagerState {
    let base_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zashboard-wsf")
        .join("ui_versions");
    build_state(base_dir)
}

#[cfg_attr(not(mobile), allow(dead_code))]
pub fn init_state_with_base_dir(base_dir: PathBuf) -> UiManagerState {
    build_state(base_dir)
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
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    let version_dir = s.base_dir.join(&tag);
    if !version_dir.join("index.html").exists() {
        return Err(format!("Version {} not found or incomplete", tag));
    }

    s.active_version = Some(tag.clone());
    write_active_version(&s.base_dir, Some(&tag));

    eprintln!("Activated upstream UI version {}", tag);
    Ok(format!("Activated version {}", tag))
}

#[tauri::command]
pub async fn ui_deactivate(
    state: State<'_, Mutex<UiManagerState>>,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    s.active_version = None;
    write_active_version(&s.base_dir, None);

    eprintln!("Deactivated upstream UI, switched to built-in");
    Ok("Switched to built-in UI".to_string())
}

#[tauri::command]
pub async fn ui_get_info(
    state: State<'_, Mutex<UiManagerState>>,
) -> Result<UiVersionInfo, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    sync_state_with_disk(&mut s);
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

// --- Protocol handler helpers ---

/// Script injected into upstream UI's index.html.
/// Adds:
///  - Floating "↩ Built-in UI" button (bottom-right, above Android nav bar)
///  - "Setup" button (top-left, above safe-area) to navigate back to upstream setup page
///  - Safe-area CSS patch for upstream zashboard's .ctrls-bar and .dock
const MANAGEMENT_BUTTON_SCRIPT: &str = r#"<script>
(function(){
  if(window.__WSF_MGMT_BTN__) return;
  window.__WSF_MGMT_BTN__=true;

  var commonStyle='z-index:2147483647;font-family:system-ui,sans-serif;cursor:pointer;'+
    'pointer-events:auto;user-select:none;-webkit-user-select:none;'+
    'border-radius:8px;backdrop-filter:blur(8px);opacity:0.9;transition:opacity 0.2s;'+
    'box-shadow:0 2px 8px rgba(0,0,0,0.3);font-size:12px;line-height:1;white-space:nowrap;';

  function clearSwAndReload(){
    function doReload(){ window.location.reload(); }
    if(!navigator.serviceWorker){ doReload(); return; }
    navigator.serviceWorker.getRegistrations().then(function(regs){
      return Promise.all(regs.map(function(r){return r.unregister();}));
    }).then(function(){
      if(!window.caches) return;
      return caches.keys().then(function(n){return Promise.all(n.map(function(k){return caches.delete(k);}));});
    }).finally(doReload);
  }

  function createContainer(){
    var c=document.createElement('div');
    c.id='wsf-mgmt-container';
    c.style.cssText='position:fixed;right:12px;bottom:calc(max(env(safe-area-inset-bottom),16px) + 8px);'+
      'z-index:2147483647;display:flex;flex-direction:column;align-items:flex-end;gap:6px;pointer-events:none;';

    // "↩ Built-in UI" button
    var btn=document.createElement('button');
    btn.id='wsf-builtin-btn';
    btn.textContent='\u21A9 Built-in UI';
    btn.style.cssText='position:relative;padding:7px 14px;'+commonStyle+
      'background:rgba(0,0,0,0.82);color:#fff;border:1px solid rgba(255,255,255,0.25);';
    btn.onmouseenter=function(){btn.style.opacity='1';};
    btn.onmouseleave=function(){btn.style.opacity='0.9';};
    btn.onclick=function(){
      btn.disabled=true; btn.textContent='Switching...'; btn.style.opacity='0.5';
      fetch('/__wsf_deactivate',{method:'POST',cache:'no-store'}).finally(clearSwAndReload);
    };
    c.appendChild(btn);

    return c;
  }

  function createSetupBtn(){
    var s=document.createElement('button');
    s.id='wsf-setup-btn';
    s.textContent='Setup';
    s.style.cssText='position:fixed;left:12px;top:calc(max(env(safe-area-inset-top),12px) + 8px);'+
      'padding:6px 10px;'+commonStyle+
      'background:rgba(255,255,255,0.62);color:#111827;border:1px solid rgba(107,114,128,0.35);';
    s.onmouseenter=function(){s.style.opacity='1';};
    s.onmouseleave=function(){s.style.opacity='0.9';};
    s.onclick=function(){ window.location.hash='#/setup'; };
    return s;
  }

  function ensure(){
    if(!document.getElementById('wsf-mgmt-container')){
      var target=document.body||document.documentElement;
      if(target) target.appendChild(createContainer());
    }
    if(!document.getElementById('wsf-setup-btn')){
      var target=document.body||document.documentElement;
      if(target) target.appendChild(createSetupBtn());
    }
  }

  var observer=new MutationObserver(function(){ ensure(); });
  function startObserver(){
    if(document.body){
      ensure();
      observer.observe(document.body,{childList:true,subtree:false});
    } else {
      document.addEventListener('DOMContentLoaded',function(){
        ensure();
        observer.observe(document.body,{childList:true,subtree:false});
      },{once:true});
    }
  }
  startObserver();
  setTimeout(ensure,300);
  setTimeout(ensure,1000);
  setTimeout(ensure,3000);
})();
</script>"#;

/// CSS patch for upstream zashboard safe-area overlap on mobile.
/// The upstream .ctrls-bar (top nav) doesn't account for env(safe-area-inset-top),
/// causing it to be covered by the Android status bar / iOS notch.
const SAFE_AREA_PATCH_STYLE: &str = r#"<style id="wsf-safe-area-patch">
:root {
  --wsf-safe-top: max(env(safe-area-inset-top), 24px);
  --wsf-safe-bottom: max(env(safe-area-inset-bottom), 24px);
}
@media (max-width: 900px) {
  .ctrls-bar { padding-top: var(--wsf-safe-top) !important; }
  .dock { bottom: calc(var(--spacing, 4px) * 2 + var(--wsf-safe-bottom)) !important; }
  .dock-shadow { height: var(--wsf-safe-bottom) !important; }
}
</style>"#;

/// Deactivate upstream UI from the protocol handler (no Tauri command context needed).
pub fn deactivate_from_protocol(state: &Mutex<UiManagerState>) {
    if let Ok(mut s) = state.lock() {
        s.active_version = None;
        write_active_version(&s.base_dir, None);
        eprintln!("Deactivated upstream UI, switched to built-in (via protocol)");
    }
}

/// Resolve a file from the active upstream version directory.
/// Returns `Some((body, mime))` if the upstream version is active and the file exists.
/// Returns `None` if no upstream version is active (caller should fall back to bundled assets).
pub fn resolve_upstream_file(state: &Mutex<UiManagerState>, path: &str) -> Option<(Vec<u8>, String)> {
    let s = state.lock().ok()?;
    let version = s.active_version.as_ref()?;
    let version_dir = s.base_dir.join(version);

    let clean_path = if path.is_empty() || path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    // Security: prevent path traversal
    if clean_path.contains("..") {
        return None;
    }

    let file_path = version_dir.join(&clean_path);

    let served_path = if file_path.exists() && file_path.is_file() {
        clean_path.as_str()
    } else {
        // SPA fallback: serve index.html for unknown paths
        "index.html"
    };

    let mut body = std::fs::read(version_dir.join(served_path)).ok()?;
    let mime = mime_type(served_path).to_string();

    // Inject management button into HTML responses
    if served_path.ends_with(".html") || served_path.ends_with(".htm") {
        body = inject_management_button(&body);
    }

    Some((body, mime))
}

/// Inject management scripts and safe-area CSS into upstream HTML.
fn inject_management_button(html_bytes: &[u8]) -> Vec<u8> {
    let html = String::from_utf8_lossy(html_bytes);
    if html.contains("__WSF_MGMT_BTN__") {
        return html_bytes.to_vec();
    }

    let mut result = html.to_string();

    // Inject safe-area CSS into <head> (before any SPA bundles run)
    if !result.contains("wsf-safe-area-patch") {
        if let Some(pos) = result.find("</head>").or_else(|| result.find("</HEAD>")) {
            result.insert_str(pos, SAFE_AREA_PATCH_STYLE);
        } else if let Some(pos) = result.find("<head>").or_else(|| result.find("<HEAD>")) {
            result.insert_str(pos + 6, SAFE_AREA_PATCH_STYLE);
        } else {
            // No <head>; prepend
            result = format!("{}{}", SAFE_AREA_PATCH_STYLE, result);
        }
    }

    // Inject management button script before </body>
    if let Some(pos) = result.find("</body>") {
        result.insert_str(pos, MANAGEMENT_BUTTON_SCRIPT);
    } else {
        result.push_str(MANAGEMENT_BUTTON_SCRIPT);
    }

    result.into_bytes()
}

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

fn sync_state_with_disk(s: &mut UiManagerState) {
    let disk_active = read_active_version(&s.base_dir);
    if disk_active.is_none() && s.active_version.is_some() {
        s.active_version = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wsf_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    // --- mime_type ---

    #[test]
    fn test_mime_type_html() {
        assert_eq!(mime_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(mime_type("page.htm"), "text/html; charset=utf-8");
    }

    #[test]
    fn test_mime_type_js_css() {
        assert_eq!(mime_type("app.js"), "application/javascript");
        assert_eq!(mime_type("module.mjs"), "application/javascript");
        assert_eq!(mime_type("style.css"), "text/css");
    }

    #[test]
    fn test_mime_type_images() {
        assert_eq!(mime_type("logo.png"), "image/png");
        assert_eq!(mime_type("photo.jpg"), "image/jpeg");
        assert_eq!(mime_type("photo.jpeg"), "image/jpeg");
        assert_eq!(mime_type("icon.svg"), "image/svg+xml");
        assert_eq!(mime_type("pic.webp"), "image/webp");
        assert_eq!(mime_type("favicon.ico"), "image/x-icon");
    }

    #[test]
    fn test_mime_type_fonts() {
        assert_eq!(mime_type("font.woff"), "font/woff");
        assert_eq!(mime_type("font.woff2"), "font/woff2");
        assert_eq!(mime_type("font.ttf"), "font/ttf");
    }

    #[test]
    fn test_mime_type_unknown() {
        assert_eq!(mime_type("data.xyz"), "application/octet-stream");
        assert_eq!(mime_type("noext"), "application/octet-stream");
    }

    // --- active version read/write ---

    #[test]
    fn test_read_write_active_version() {
        let dir = create_temp_dir();

        assert_eq!(read_active_version(&dir), None);

        write_active_version(&dir, Some("v1.0.0"));
        assert_eq!(read_active_version(&dir), Some("v1.0.0".to_string()));

        write_active_version(&dir, None);
        assert_eq!(read_active_version(&dir), None);

        cleanup(&dir);
    }

    #[test]
    fn test_read_active_version_builtin() {
        let dir = create_temp_dir();
        fs::write(dir.join(ACTIVE_VERSION_FILE), "builtin").unwrap();
        assert_eq!(read_active_version(&dir), None);
        cleanup(&dir);
    }

    // --- build_state ---

    #[test]
    fn test_build_state_empty_dir() {
        let dir = create_temp_dir();
        let state = build_state(dir.clone());
        assert!(state.active_version.is_none());
        assert_eq!(state.base_dir, dir);
        assert!(state.custom_releases_url.is_none());
        assert!(state.custom_download_base.is_none());
        cleanup(&dir);
    }

    #[test]
    fn test_build_state_with_active_version() {
        let dir = create_temp_dir();
        let ver_dir = dir.join("v1.2.3");
        fs::create_dir_all(&ver_dir).unwrap();
        fs::write(ver_dir.join("index.html"), "<html></html>").unwrap();
        write_active_version(&dir, Some("v1.2.3"));

        let state = build_state(dir.clone());
        assert_eq!(state.active_version, Some("v1.2.3".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn test_build_state_resets_missing_version() {
        let dir = create_temp_dir();
        write_active_version(&dir, Some("v_gone"));
        // No version dir on disk

        let state = build_state(dir.clone());
        assert!(state.active_version.is_none());
        // Also check that disk file was reset
        assert_eq!(read_active_version(&dir), None);
        cleanup(&dir);
    }

    // --- resolve_upstream_file ---

    #[test]
    fn test_resolve_upstream_file_no_active() {
        let dir = create_temp_dir();
        let state = Mutex::new(UiManagerState {
            active_version: None,
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        });
        assert!(resolve_upstream_file(&state, "/").is_none());
        cleanup(&dir);
    }

    #[test]
    fn test_resolve_upstream_file_serves_index() {
        let dir = create_temp_dir();
        let ver_dir = dir.join("v1.0.0");
        fs::create_dir_all(&ver_dir).unwrap();
        fs::write(ver_dir.join("index.html"), "<html><body>hello</body></html>").unwrap();

        let state = Mutex::new(UiManagerState {
            active_version: Some("v1.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        });

        let result = resolve_upstream_file(&state, "/");
        assert!(result.is_some());
        let (body, mime) = result.unwrap();
        let html = String::from_utf8_lossy(&body);
        // Original content is preserved
        assert!(html.contains("hello"));
        // Management button script is injected
        assert!(html.contains("__WSF_MGMT_BTN__"));
        assert!(html.contains("wsf-mgmt-btn"));
        assert_eq!(mime, "text/html; charset=utf-8");
        cleanup(&dir);
    }

    #[test]
    fn test_resolve_upstream_file_serves_asset() {
        let dir = create_temp_dir();
        let ver_dir = dir.join("v1.0.0");
        let assets_dir = ver_dir.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        fs::write(ver_dir.join("index.html"), "<html></html>").unwrap();
        fs::write(assets_dir.join("app.js"), "console.log('hi')").unwrap();

        let state = Mutex::new(UiManagerState {
            active_version: Some("v1.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        });

        let result = resolve_upstream_file(&state, "/assets/app.js");
        assert!(result.is_some());
        let (body, mime) = result.unwrap();
        assert_eq!(String::from_utf8_lossy(&body), "console.log('hi')");
        assert_eq!(mime, "application/javascript");
        cleanup(&dir);
    }

    #[test]
    fn test_resolve_upstream_file_spa_fallback() {
        let dir = create_temp_dir();
        let ver_dir = dir.join("v1.0.0");
        fs::create_dir_all(&ver_dir).unwrap();
        fs::write(ver_dir.join("index.html"), "<html><body>spa</body></html>").unwrap();

        let state = Mutex::new(UiManagerState {
            active_version: Some("v1.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        });

        // Non-existent path should fall back to index.html
        let result = resolve_upstream_file(&state, "/some/route");
        assert!(result.is_some());
        let (body, mime) = result.unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(html.contains("spa"));
        assert!(html.contains("__WSF_MGMT_BTN__"));
        assert_eq!(mime, "text/html; charset=utf-8");
        cleanup(&dir);
    }

    #[test]
    fn test_resolve_upstream_file_blocks_traversal() {
        let dir = create_temp_dir();
        let ver_dir = dir.join("v1.0.0");
        fs::create_dir_all(&ver_dir).unwrap();
        fs::write(ver_dir.join("index.html"), "<html></html>").unwrap();

        let state = Mutex::new(UiManagerState {
            active_version: Some("v1.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        });

        assert!(resolve_upstream_file(&state, "/../../etc/passwd").is_none());
        assert!(resolve_upstream_file(&state, "/../secret").is_none());
        cleanup(&dir);
    }

    // --- sync_state_with_disk ---

    #[test]
    fn test_sync_state_with_disk_clears_stale() {
        let dir = create_temp_dir();
        // No active file on disk, but state thinks v1 is active
        let mut state = UiManagerState {
            active_version: Some("v1.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        };

        sync_state_with_disk(&mut state);
        assert!(state.active_version.is_none());
        cleanup(&dir);
    }

    #[test]
    fn test_sync_state_with_disk_keeps_valid() {
        let dir = create_temp_dir();
        write_active_version(&dir, Some("v2.0.0"));
        let mut state = UiManagerState {
            active_version: Some("v2.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        };

        sync_state_with_disk(&mut state);
        assert_eq!(state.active_version, Some("v2.0.0".to_string()));
        cleanup(&dir);
    }

    // --- custom URL read/write ---

    #[test]
    fn test_custom_url_read_write() {
        let dir = create_temp_dir();

        assert!(read_custom_url(&dir, CUSTOM_RELEASES_URL_FILE).is_none());

        write_custom_url(&dir, CUSTOM_RELEASES_URL_FILE, Some("https://example.com/releases"));
        assert_eq!(
            read_custom_url(&dir, CUSTOM_RELEASES_URL_FILE),
            Some("https://example.com/releases".to_string())
        );

        write_custom_url(&dir, CUSTOM_RELEASES_URL_FILE, None);
        assert!(read_custom_url(&dir, CUSTOM_RELEASES_URL_FILE).is_none());

        cleanup(&dir);
    }

    // --- dir_size ---

    #[test]
    fn test_dir_size() {
        let dir = create_temp_dir();
        fs::write(dir.join("a.txt"), "hello").unwrap(); // 5 bytes
        fs::write(dir.join("b.txt"), "world!").unwrap(); // 6 bytes
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("c.txt"), "12345678").unwrap(); // 8 bytes

        assert_eq!(dir_size(&dir), 19);
        cleanup(&dir);
    }

    // --- inject_management_button ---

    #[test]
    fn test_inject_management_button_before_body_close() {
        let html = b"<html><body>content</body></html>";
        let result = inject_management_button(html);
        let result_str = String::from_utf8_lossy(&result);
        assert!(result_str.contains("__WSF_MGMT_BTN__"));
        assert!(result_str.contains("content"));
        // Script should be before </body>
        let script_pos = result_str.find("__WSF_MGMT_BTN__").unwrap();
        let body_close_pos = result_str.find("</body>").unwrap();
        assert!(script_pos < body_close_pos);
    }

    #[test]
    fn test_inject_management_button_no_body_tag() {
        let html = b"<html>no body tag</html>";
        let result = inject_management_button(html);
        let result_str = String::from_utf8_lossy(&result);
        assert!(result_str.contains("__WSF_MGMT_BTN__"));
        assert!(result_str.contains("no body tag"));
    }

    #[test]
    fn test_inject_management_button_idempotent() {
        let html = b"<html><body>test</body></html>";
        let first = inject_management_button(html);
        let second = inject_management_button(&first);
        // Should not double-inject
        assert_eq!(first, second);
    }

    #[test]
    fn test_js_assets_not_injected() {
        let dir = create_temp_dir();
        let ver_dir = dir.join("v1.0.0");
        let assets_dir = ver_dir.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        fs::write(ver_dir.join("index.html"), "<html><body></body></html>").unwrap();
        fs::write(assets_dir.join("app.js"), "console.log('hi')").unwrap();

        let state = Mutex::new(UiManagerState {
            active_version: Some("v1.0.0".to_string()),
            base_dir: dir.clone(),
            custom_releases_url: None,
            custom_download_base: None,
        });

        // JS files should NOT get the management button injected
        let result = resolve_upstream_file(&state, "/assets/app.js");
        assert!(result.is_some());
        let (body, _) = result.unwrap();
        assert_eq!(String::from_utf8_lossy(&body), "console.log('hi')");
        assert!(!String::from_utf8_lossy(&body).contains("__WSF_MGMT_BTN__"));
        cleanup(&dir);
    }
}

