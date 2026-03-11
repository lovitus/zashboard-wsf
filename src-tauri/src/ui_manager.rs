use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
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
}

#[derive(Debug, Clone, Serialize)]
pub struct UiVersionInfo {
    pub active_version: Option<String>,
    pub downloaded_versions: Vec<DownloadedVersion>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadedVersion {
    pub tag: String,
    pub size_bytes: u64,
}

const ACTIVE_VERSION_FILE: &str = "ui_active_version.txt";
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/Zephyruso/zashboard/releases";

fn read_active_version(base_dir: &PathBuf) -> Option<String> {
    let path = base_dir.join(ACTIVE_VERSION_FILE);
    std::fs::read_to_string(&path).ok().and_then(|s| {
        let s = s.trim().to_string();
        if s.is_empty() || s == "builtin" {
            None
        } else {
            Some(s)
        }
    })
}

fn write_active_version(base_dir: &PathBuf, version: Option<&str>) {
    let path = base_dir.join(ACTIVE_VERSION_FILE);
    let content = version.unwrap_or("builtin");
    let _ = std::fs::write(&path, content);
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

pub fn init_state() -> UiManagerState {
    let base_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zashboard-wsf")
        .join("ui_versions");
    std::fs::create_dir_all(&base_dir).ok();

    let active_version = read_active_version(&base_dir);
    eprintln!(
        "UI manager: base_dir={}, active={:?}",
        base_dir.display(),
        active_version
    );

    UiManagerState {
        active_version,
        base_dir,
    }
}

// --- Commands ---

#[tauri::command]
pub async fn ui_fetch_releases() -> Result<Vec<UpstreamRelease>, String> {
    let client = reqwest::Client::builder()
        .user_agent("zashboard-wsf")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(GITHUB_RELEASES_URL)
        .send()
        .await
        .map_err(|e| format!("Fetch failed: {}", e))?;

    let releases: Vec<UpstreamRelease> = response
        .json::<Vec<UpstreamRelease>>()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    Ok(releases)
}

#[tauri::command]
pub async fn ui_download_version(
    state: State<'_, Mutex<UiManagerState>>,
    tag: String,
) -> Result<String, String> {
    let base_dir = {
        let s = state.lock().map_err(|e| e.to_string())?;
        s.base_dir.clone()
    };

    let version_dir = base_dir.join(&tag);
    if version_dir.join("index.html").exists() {
        return Ok(format!("Version {} already downloaded", tag));
    }

    // Download dist.zip from GitHub release
    let url = format!(
        "https://github.com/Zephyruso/zashboard/releases/download/{}/dist.zip",
        tag
    );

    eprintln!("Downloading upstream UI: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("zashboard-wsf")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Read failed: {}", e))?;

    eprintln!("Downloaded {} bytes, extracting...", bytes.len());

    // Extract zip
    std::fs::create_dir_all(&version_dir).map_err(|e| format!("Create dir failed: {}", e))?;

    let reader = std::io::Cursor::new(bytes.as_ref());
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("Zip open failed: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;

        let name = file.name().to_string();

        // The zip contains files under "dist/" prefix - strip it
        let relative = if let Some(stripped) = name.strip_prefix("dist/") {
            stripped
        } else {
            &name
        };

        if relative.is_empty() {
            continue;
        }

        let out_path = version_dir.join(relative);

        if file.is_dir() {
            std::fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut out_file =
                std::fs::File::create(&out_path).map_err(|e| format!("File create failed: {}", e))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| format!("File write failed: {}", e))?;
        }
    }

    eprintln!("Extracted upstream UI version {} to {}", tag, version_dir.display());
    Ok(format!("Downloaded version {}", tag))
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

    // Navigate main webview to the custom protocol
    if let Some(webview) = app.get_webview_window("main") {
        let _ = webview.eval("window.location.href = 'zui://localhost/'");
    }

    Ok(format!("Activated version {}", tag))
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

    // Navigate back to built-in UI
    // Try platform-appropriate URL
    if let Some(webview) = app.get_webview_window("main") {
        let _ = webview.eval(
            r#"
            try { window.location.href = 'tauri://localhost/'; }
            catch(e) { window.location.href = 'https://tauri.localhost/'; }
            "#,
        );
    }

    Ok("Deactivated UI override".to_string())
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
    })
}

#[tauri::command]
pub async fn ui_delete_version(
    state: State<'_, Mutex<UiManagerState>>,
    tag: String,
) -> Result<String, String> {
    let base_dir = {
        let s = state.lock().map_err(|e| e.to_string())?;
        if s.active_version.as_deref() == Some(tag.as_str()) {
            return Err("Cannot delete the currently active version".to_string());
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
    if(window.__TAURI_INTERNALS__){
      window.__TAURI_INTERNALS__.invoke('ui_deactivate').catch(function(){});
    }
    setTimeout(function(){
      try{window.location.href='tauri://localhost/'}
      catch(e){try{window.location.href='https://tauri.localhost/'}catch(e2){}}
    },100);
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
                .body(b"State lock error".to_vec())
                .unwrap();
        }
    };

    let active = match &s.active_version {
        Some(v) => v.clone(),
        None => {
            return tauri::http::Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(b"No active UI override. Return to built-in UI.".to_vec())
                .unwrap();
        }
    };

    let version_dir = s.base_dir.join(&active);
    drop(s);

    // Parse path from URI
    let uri = request.uri().to_string();
    // Handle various URI formats: zui://localhost/path, zui://localhost, zui:///path
    let path = if let Some(rest) = uri.strip_prefix("zui://localhost") {
        if rest.is_empty() { "/" } else { rest }
    } else if let Some(rest) = uri.strip_prefix("zui://") {
        if rest.is_empty() { "/" } else { rest }
    } else {
        "/"
    };

    // Strip query string
    let path = path.split('?').next().unwrap_or("/");
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
            let modified = html.replace("</body>", &format!("{}</body>", RETURN_BUTTON_SCRIPT));
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
            let modified = html.replace("</body>", &format!("{}</body>", RETURN_BUTTON_SCRIPT));
            (modified.into_bytes(), "text/html; charset=utf-8")
        } else {
            return tauri::http::Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(b"Not Found".to_vec())
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
