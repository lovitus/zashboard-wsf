use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, State};
use tokio::process::{Child, Command};

/// Tunnel configuration for a backend that needs port forwarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub id: String,
    pub backend_uuid: String,
    /// "gust", "slider", or "flyssh"
    pub tool: String,
    /// Full command-line arguments (e.g., ["-L", "127.0.0.1:19090:127.0.0.1:9090", "user@host"])
    pub args: Vec<String>,
    /// Local port that will be forwarded to
    pub local_port: u16,
    /// Whether to auto-start when app launches
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
}

impl Default for TunnelState {
    fn default() -> Self {
        Self {
            processes: HashMap::new(),
            configs: Vec::new(),
        }
    }
}

fn get_config_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
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
    let mut statuses = Vec::new();

    for config in &s.configs {
        let running = if let Some(child) = s.processes.get_mut(&config.id) {
            match child.try_wait() {
                Ok(Some(_)) => false, // exited
                Ok(None) => true,     // still running
                Err(_) => false,
            }
        } else {
            false
        };

        let pid = if running {
            s.processes.get(&config.id).and_then(|c| c.id())
        } else {
            // Clean up dead process
            s.processes.remove(&config.id);
            None
        };

        statuses.push(TunnelStatus {
            id: config.id.clone(),
            running,
            pid,
            error: None,
        });
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
    let mut s = state.lock().map_err(|e| e.to_string())?;

    // Kill process if running
    if let Some(mut child) = s.processes.remove(&id) {
        child.kill().await.ok();
    }

    s.configs.retain(|c| c.id != id);
    save_configs(&s.configs);
    Ok(())
}

#[tauri::command]
async fn start_tunnel(
    id: String,
    state: State<'_, Mutex<TunnelState>>,
) -> Result<TunnelStatus, String> {
    let (tool, args) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        let config = s
            .configs
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("Tunnel {} not found", id))?;
        (config.tool.clone(), config.args.clone())
    };

    let child = Command::new(&tool)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", tool, e))?;

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
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = s.processes.remove(&id) {
        child.kill().await.ok();
    }
    Ok(TunnelStatus {
        id,
        running: false,
        pid: None,
        error: None,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tunnel_state = TunnelState::default();
    tunnel_state.configs = load_configs();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(Mutex::new(tunnel_state))
        .invoke_handler(tauri::generate_handler![
            get_tunnels,
            get_tunnel_statuses,
            save_tunnel,
            remove_tunnel,
            start_tunnel,
            stop_tunnel,
        ])
        .setup(|app| {
            // Auto-start tunnels marked for auto-start
            let state = app.state::<Mutex<TunnelState>>();
            let auto_start_ids: Vec<String> = {
                let s = state.lock().unwrap();
                s.configs
                    .iter()
                    .filter(|c| c.auto_start)
                    .map(|c| c.id.clone())
                    .collect()
            };

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Small delay to let the app window load
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                for id in auto_start_ids {
                    let state = handle.state::<Mutex<TunnelState>>();
                    let (tool, args) = {
                        let s = state.lock().unwrap();
                        match s.configs.iter().find(|c| c.id == id) {
                            Some(c) => (c.tool.clone(), c.args.clone()),
                            None => continue,
                        }
                    };

                    match Command::new(&tool)
                        .args(&args)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .kill_on_drop(true)
                        .spawn()
                    {
                        Ok(child) => {
                            let mut s = state.lock().unwrap();
                            s.processes.insert(id, child);
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
