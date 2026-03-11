use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
#[cfg(desktop)]
use tauri::menu::{MenuBuilder, MenuItemBuilder};
#[cfg(desktop)]
use tauri::tray::TrayIconBuilder;
#[cfg(desktop)]
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri::{Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

// --- Windows Job Object: auto-kill ONLY our child processes on exit ---
// When the main process exits (gracefully or force-killed by installer),
// the OS closes the Job Object handle and terminates all assigned children.
// Other gust/slider instances running outside this app are NOT affected.
#[cfg(windows)]
mod job_object {
    use std::sync::OnceLock;

    static JOB: OnceLock<usize> = OnceLock::new();

    extern "system" {
        fn CreateJobObjectW(attrs: *const u8, name: *const u16) -> usize;
        fn SetInformationJobObject(job: usize, class: u32, info: *const u8, len: u32) -> i32;
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> usize;
        fn AssignProcessToJobObject(job: usize, process: usize) -> i32;
        fn CloseHandle(handle: usize) -> i32;
    }

    #[repr(C)]
    struct BasicLimitInfo {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        min_working_set: usize,
        max_working_set: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    struct IoCounters {
        read_ops: u64, write_ops: u64, other_ops: u64,
        read_bytes: u64, write_bytes: u64, other_bytes: u64,
    }

    #[repr(C)]
    struct ExtendedLimitInfo {
        basic: BasicLimitInfo,
        io: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory: usize,
        peak_job_memory: usize,
    }

    pub fn init() {
        const KILL_ON_JOB_CLOSE: u32 = 0x2000;
        const INFO_CLASS_EXTENDED: u32 = 9;

        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job == 0 {
                eprintln!("WARNING: Failed to create Job Object");
                return;
            }

            let mut info: ExtendedLimitInfo = std::mem::zeroed();
            info.basic.limit_flags = KILL_ON_JOB_CLOSE;

            let ok = SetInformationJobObject(
                job, INFO_CLASS_EXTENDED,
                &info as *const _ as *const u8,
                std::mem::size_of::<ExtendedLimitInfo>() as u32,
            );
            if ok == 0 {
                eprintln!("WARNING: Failed to configure Job Object");
                CloseHandle(job);
                return;
            }

            JOB.set(job).ok();
            eprintln!("Job Object created — child processes will auto-terminate on app exit");
        }
    }

    pub fn assign(pid: u32) {
        let Some(&job) = JOB.get() else { return };
        const PROCESS_SET_QUOTA: u32 = 0x0100;
        const PROCESS_TERMINATE: u32 = 0x0001;

        unsafe {
            let handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
            if handle == 0 {
                eprintln!("WARNING: Cannot open child pid {} for job assignment", pid);
                return;
            }
            let ok = AssignProcessToJobObject(job, handle);
            CloseHandle(handle);
            if ok == 0 {
                eprintln!("WARNING: Cannot assign pid {} to job", pid);
            }
        }
    }
}

/// Tunnel configuration for a backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub backend_uuid: String,
    /// "gust" or "slider"
    pub tool: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub local_port: u16,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub id: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
    pub logs: Vec<String>,
}

struct TunnelProcess {
    child: Child,
    logs: Arc<Mutex<VecDeque<String>>>,
}

struct TunnelState {
    processes: HashMap<String, TunnelProcess>,
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
    // On Android, sidecar binaries are packaged as lib*.so in jniLibs
    // and extracted to the native library directory by the OS.
    #[cfg(target_os = "android")]
    {
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                if let Some(path_str) = line.split_whitespace().last() {
                    if path_str.ends_with(".so") && path_str.contains("/lib/") {
                        let path = PathBuf::from(path_str);
                        if let Some(parent) = path.parent() {
                            eprintln!("Android native lib dir: {}", parent.display());
                            return parent.to_path_buf();
                        }
                    }
                }
            }
        }
        eprintln!("WARNING: Could not detect Android native lib dir from /proc/self/maps");
    }

    // Desktop: sidecars are next to the main executable
    // In dev: they're in src-tauri/binaries/ with target triple suffix
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    PathBuf::from(".")
}

fn resolve_tool_path(sidecar_dir: &PathBuf, tool: &str) -> PathBuf {
    // On Android, binaries are named lib<tool>.so in the native lib dir
    #[cfg(target_os = "android")]
    {
        let android_path = sidecar_dir.join(format!("lib{}.so", tool));
        if android_path.exists() {
            return android_path;
        }
    }

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

fn spawn_tunnel(sidecar_dir: &PathBuf, tool: &str, args: &[String]) -> Result<TunnelProcess, String> {
    let tool_path = resolve_tool_path(sidecar_dir, tool);
    eprintln!("Starting tunnel: {} {:?}", tool_path.display(), args);

    let mut cmd = Command::new(&tool_path);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start {}: {}", tool_path.display(), e))?;

    // Assign to Job Object so this child is auto-killed when our app exits
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        job_object::assign(pid);
    }

    let logs: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::with_capacity(12)));

    // Spawn stdout reader so the process doesn't block on full pipe buffer
    if let Some(stdout) = child.stdout.take() {
        let logs_clone = logs.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut buf = logs_clone.lock().unwrap();
                if buf.len() >= 10 {
                    buf.pop_front();
                }
                buf.push_back(line);
            }
        });
    }

    // Spawn stderr reader
    if let Some(stderr) = child.stderr.take() {
        let logs_clone = logs.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut buf = logs_clone.lock().unwrap();
                if buf.len() >= 10 {
                    buf.pop_front();
                }
                buf.push_back(line);
            }
        });
    }

    Ok(TunnelProcess { child, logs })
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
        if let Some(process) = s.processes.get_mut(id) {
            let running = match process.child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            };

            let pid = if running {
                process.child.id()
            } else {
                dead_ids.push(id.clone());
                None
            };

            let logs: Vec<String> = process.logs.lock().unwrap().iter().cloned().collect();

            statuses.push(TunnelStatus {
                id: id.clone(),
                running,
                pid,
                error: None,
                logs,
            });
        } else {
            statuses.push(TunnelStatus {
                id: id.clone(),
                running: false,
                pid: None,
                error: None,
                logs: Vec::new(),
            });
        }
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
    let process_to_kill;
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        process_to_kill = s.processes.remove(&id);
        s.configs.retain(|c| c.id != id);
        save_configs(&s.configs);
    }

    if let Some(mut process) = process_to_kill {
        process.child.kill().await.ok();
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

    let process = spawn_tunnel(&sidecar_dir, &tool, &args)?;
    let pid = process.child.id();

    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.processes.insert(id.clone(), process);
    }

    Ok(TunnelStatus {
        id,
        running: true,
        pid,
        error: None,
        logs: Vec::new(),
    })
}

#[tauri::command]
async fn stop_tunnel(
    id: String,
    state: State<'_, Mutex<TunnelState>>,
) -> Result<TunnelStatus, String> {
    let process_to_kill = {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.processes.remove(&id)
    };

    if let Some(mut process) = process_to_kill {
        process.child.kill().await.ok();
    }

    Ok(TunnelStatus {
        id,
        running: false,
        pid: None,
        error: None,
        logs: Vec::new(),
    })
}

#[tauri::command]
async fn add_defender_exclusion(state: State<'_, Mutex<TunnelState>>) -> Result<String, String> {
    let sidecar_dir = {
        let s = state.lock().map_err(|e| e.to_string())?;
        s.sidecar_dir.clone()
    };

    let dir_str = sidecar_dir.to_string_lossy().to_string();
    eprintln!("Adding Defender exclusion for: {}", dir_str);

    // Write a temp .ps1 script to avoid nested quoting issues
    let temp_script = std::env::temp_dir().join("zashboard-av-fix.ps1");
    let script_content = format!("Add-MpPreference -ExclusionPath '{}'\n", dir_str);
    std::fs::write(&temp_script, &script_content)
        .map_err(|e| format!("Failed to write temp script: {}", e))?;

    // Use Start-Process -Verb RunAs to trigger UAC elevation
    let output = tokio::process::Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            &format!(
                "Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"' -Verb RunAs -Wait",
                temp_script.display()
            ),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    // Clean up temp script
    let _ = std::fs::remove_file(&temp_script);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Exclusion failed (UAC denied?): {}", stderr.trim()));
    }

    Ok(dir_str)
}

#[cfg(desktop)]
fn show_or_create_window(handle: &tauri::AppHandle) {
    if let Some(win) = handle.get_webview_window("main") {
        win.show().ok();
        win.set_focus().ok();
    } else {
        let _ = WebviewWindowBuilder::new(
            handle,
            "main",
            WebviewUrl::App("index.html".into()),
        )
        .title("Zashboard - Mihomo Dashboard")
        .inner_size(1200.0, 800.0)
        .min_inner_size(400.0, 600.0)
        .build();
    }
}

// --- App entry ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    job_object::init();

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
            add_defender_exclusion,
        ])
        .setup(move |app| {
            // --- System Tray (desktop only) ---
            #[cfg(desktop)]
            {
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
                                show_or_create_window(app);
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                        if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                            show_or_create_window(tray.app_handle());
                        }
                    })
                    .build(app)?;
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
                        Ok(process) => {
                            let state = handle.state::<Mutex<TunnelState>>();
                            let mut s = state.lock().unwrap();
                            s.processes.insert(id.clone(), process);
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            #[cfg(desktop)]
            if let tauri::RunEvent::ExitRequested { api, .. } = &event {
                api.prevent_exit();
            }
        });
}
