use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, State};
use tokio::process::{Child, Command};

/// Tunnel configuration for a backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub id: String,
    pub backend_uuid: String,
    /// "gust" or "slider"
    pub tool: String,
    pub args: Vec<String>,
    pub local_port: u16,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub id: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
}

struct TunnelState {
    processes: HashMap<String, Child>,
    configs: Vec<TunnelConfig>,
    sidecar_dir: PathBuf,
}

// --- Persistence ---

fn get_config_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zashboard");
    std::fs::create_dir_all(&dir).ok();
    dir.join("tunnels.json")
}

fn load_configs() -> Vec<TunnelConfig> {
    let path = get_config_path();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_configs(configs: &[TunnelConfig]) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(configs) {
        std::fs::write(path, data).ok();
    }
}

// --- Sidecar resolution ---

fn resolve_sidecar_dir() -> PathBuf {
    // In release: sidecars are next to the main executable
    // In dev: they're in src-tauri/binaries/ with target triple suffix
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    PathBuf::from(".")
}

fn resolve_tool_path(sidecar_dir: &PathBuf, tool: &str) -> PathBuf {
    // Try exact name first (release mode: gust.exe / slider.exe next to binary)
    let base = sidecar_dir.join(tool);
    #[cfg(windows)]
    {
        let with_ext = base.with_extension("exe");
        if with_ext.exists() {
            return with_ext;
        }
    }
    #[cfg(not(windows))]
    {
        if base.exists() {
            return base;
        }
    }

    // Try with target triple suffix (dev mode)
    let target = if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        "x86_64-pc-windows-msvc"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "aarch64-apple-darwin"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "x86_64-apple-darwin"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        ""
    };

    if !target.is_empty() {
        let name = format!("{}-{}", tool, target);
        let suffixed = sidecar_dir.join(&name);
        #[cfg(windows)]
        {
            let with_ext = suffixed.with_extension("exe");
            if with_ext.exists() {
                return with_ext;
            }
        }
        #[cfg(not(windows))]
        {
            if suffixed.exists() {
                return suffixed;
            }
        }
    }

    // Fallback: just return the base name, will fail with clear error
    base
}

fn spawn_tunnel(sidecar_dir: &PathBuf, tool: &str, args: &[String]) -> Result<Child, String> {
    let tool_path = resolve_tool_path(sidecar_dir, tool);
    eprintln!("Starting tunnel: {} {:?}", tool_path.display(), args);

    Command::new(&tool_path)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", tool_path.display(), e))
}

// --- Tauri Commands ---

#[tauri::command]
async fn get_tunnels(state: State<'_, Mutex<TunnelState>>) -> Result<Vec<TunnelConfig>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.configs.clone())
}

#[tauri::command]
async fn get_tunnel_statuses(
    state: State<'_, Mutex<TunnelState>>,
) -> Result<Vec<TunnelStatus>, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    let ids: Vec<String> = s.configs.iter().map(|c| c.id.clone()).collect();
    let mut statuses = Vec::new();
    let mut dead_ids = Vec::new();

    for id in &ids {
        let running = if let Some(child) = s.processes.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        };

        let pid = if running {
            s.processes.get(id).and_then(|c| c.id())
        } else {
            dead_ids.push(id.clone());
            None
        };

        statuses.push(TunnelStatus {
            id: id.clone(),
            running,
            pid,
            error: None,
        });
    }

    for id in dead_ids {
        s.processes.remove(&id);
    }

    Ok(statuses)
}

#[tauri::command]
async fn save_tunnel(
    config: TunnelConfig,
    state: State<'_, Mutex<TunnelState>>,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(existing) = s.configs.iter_mut().find(|c| c.id == config.id) {
        *existing = config;
    } else {
        s.configs.push(config);
    }
    save_configs(&s.configs);
    Ok(())
}

#[tauri::command]
async fn remove_tunnel(
    id: String,
    state: State<'_, Mutex<TunnelState>>,
) -> Result<(), String> {
    let child_to_kill;
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        child_to_kill = s.processes.remove(&id);
        s.configs.retain(|c| c.id != id);
        save_configs(&s.configs);
    }

    if let Some(mut child) = child_to_kill {
        child.kill().await.ok();
    }

    Ok(())
}

#[tauri::command]
async fn start_tunnel(
    id: String,
    state: State<'_, Mutex<TunnelState>>,
) -> Result<TunnelStatus, String> {
    let (sidecar_dir, tool, args) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        let config = s
            .configs
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("Tunnel {} not found", id))?;
        (s.sidecar_dir.clone(), config.tool.clone(), config.args.clone())
    };

    let child = spawn_tunnel(&sidecar_dir, &tool, &args)?;
    let pid = child.id();

    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.processes.insert(id.clone(), child);
    }

    Ok(TunnelStatus {
        id,
        running: true,
        pid,
        error: None,
    })
}

#[tauri::command]
async fn stop_tunnel(
    id: String,
    state: State<'_, Mutex<TunnelState>>,
) -> Result<TunnelStatus, String> {
    let child_to_kill = {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.processes.remove(&id)
    };

    if let Some(mut child) = child_to_kill {
        child.kill().await.ok();
    }

    Ok(TunnelStatus {
        id,
        running: false,
        pid: None,
        error: None,
    })
}

// --- App entry ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let sidecar_dir = resolve_sidecar_dir();

    // Pre-check sidecar binaries exist
    for tool in &["gust", "slider"] {
        let path = resolve_tool_path(&sidecar_dir, tool);
        if path.exists() {
            eprintln!("Sidecar found: {} -> {}", tool, path.display());
        } else {
            eprintln!("WARNING: Sidecar not found: {} (looked at {})", tool, path.display());
        }
    }

    let ts = TunnelState {
        processes: HashMap::new(),
        configs: load_configs(),
        sidecar_dir: sidecar_dir.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(Mutex::new(ts))
        .invoke_handler(tauri::generate_handler![
            get_tunnels,
            get_tunnel_statuses,
            save_tunnel,
            remove_tunnel,
            start_tunnel,
            stop_tunnel,
        ])
        .setup(move |app| {
            // --- System Tray ---
            let show_item = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Zashboard")
                .menu(&tray_menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                win.show().ok();
                                win.set_focus().ok();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            win.show().ok();
                            win.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            // Hide to tray on window close instead of quitting
            if let Some(win) = app.get_webview_window("main") {
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        win_clone.hide().ok();
                    }
                });
            }

            // --- Auto-start tunnels ---
            let state = app.state::<Mutex<TunnelState>>();
            let auto_start_configs: Vec<(String, String, Vec<String>)> = {
                let s = state.lock().unwrap();
                s.configs
                    .iter()
                    .filter(|c| c.auto_start)
                    .map(|c| (c.id.clone(), c.tool.clone(), c.args.clone()))
                    .collect()
            };

            let sidecar_dir_clone = sidecar_dir.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                for (id, tool, args) in auto_start_configs {
                    match spawn_tunnel(&sidecar_dir_clone, &tool, &args) {
                        Ok(child) => {
                            let state = handle.state::<Mutex<TunnelState>>();
                            let mut s = state.lock().unwrap();
                            s.processes.insert(id.clone(), child);
                            eprintln!("Auto-started tunnel: {}", id);
                        }
                        Err(e) => {
                            eprintln!("Auto-start tunnel {} failed: {}", id, e);
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
