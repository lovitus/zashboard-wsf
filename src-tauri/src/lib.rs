mod ui_manager;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
#[cfg(desktop)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(desktop)]
use tauri::menu::{MenuBuilder, MenuItemBuilder};
#[cfg(desktop)]
use tauri::tray::TrayIconBuilder;
#[cfg(desktop)]
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri::{Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

#[cfg(desktop)]
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

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
    config_path: PathBuf,
    running_set: HashSet<String>,
}

// --- Persistence ---

fn get_config_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zashboard");
    std::fs::create_dir_all(&dir).ok();
    dir.join("tunnels.json")
}

fn load_configs(path: &PathBuf) -> Vec<TunnelConfig> {
    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(data) => {
                let configs: Vec<TunnelConfig> = serde_json::from_str(&data).unwrap_or_default();
                eprintln!("Loaded {} tunnel configs from {}", configs.len(), path.display());
                configs
            }
            Err(e) => {
                eprintln!("WARNING: Failed to read config {}: {}", path.display(), e);
                Vec::new()
            }
        }
    } else {
        eprintln!("No config file at {}, starting fresh", path.display());
        Vec::new()
    }
}

fn save_configs(configs: &[TunnelConfig], path: &PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    match serde_json::to_string_pretty(configs) {
        Ok(data) => {
            if let Err(e) = std::fs::write(path, &data) {
                eprintln!("WARNING: Failed to save configs to {}: {}", path.display(), e);
            } else {
                eprintln!("Saved {} tunnel configs to {}", configs.len(), path.display());
            }
        }
        Err(e) => eprintln!("WARNING: Failed to serialize configs: {}", e),
    }
}

// --- Embedded sidecar binaries (gzip-compressed) ---
// CI places real compressed binaries in src-tauri/embed/
// build.rs creates empty placeholders for dev builds
const GUST_GZ: &[u8] = include_bytes!("../embed/gust.gz");
const SLIDER_GZ: &[u8] = include_bytes!("../embed/slider.gz");

#[allow(dead_code)]
fn get_sidecar_cache_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zashboard")
        .join("bin");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn fnv1a_hash(data: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

fn extract_embedded_binary(name: &str, compressed: &[u8], target_dir: &PathBuf) -> Result<PathBuf, String> {
    if compressed.is_empty() {
        return Err(format!("No embedded {} binary (dev mode)", name));
    }

    let filename = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };
    let target = target_dir.join(&filename);

    // Hash-based version check: skip re-extraction if already up-to-date
    let hash = fnv1a_hash(compressed);
    let marker = target_dir.join(format!(".{}.hash", name));
    if target.exists() {
        if let Ok(stored) = std::fs::read_to_string(&marker) {
            if stored.trim() == hash {
                return Ok(target);
            }
        }
    }

    // Decompress gzip and write
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(compressed);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)
        .map_err(|e| format!("Failed to decompress {}: {}", name, e))?;

    std::fs::write(&target, &data)
        .map_err(|e| format!("Failed to write {} (antivirus blocking?): {}", target.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755)).ok();
    }

    std::fs::write(&marker, &hash).ok();
    eprintln!("Extracted embedded binary: {}", target.display());
    Ok(target)
}

fn ensure_sidecar_available(sidecar_dir: &PathBuf, tool: &str) -> Result<(), String> {
    let tool_path = resolve_tool_path(sidecar_dir, tool);
    if tool_path.exists() {
        return Ok(());
    }

    let data = match tool {
        "gust" => GUST_GZ,
        "slider" => SLIDER_GZ,
        _ => return Err(format!("Unknown tool: {}", tool)),
    };

    extract_embedded_binary(tool, data, sidecar_dir).map(|_| ())
}

// --- Sidecar resolution ---

fn resolve_sidecar_dir() -> PathBuf {
    // On Android, sidecar binaries are packaged as lib*.so in jniLibs
    // and extracted to the native library directory by the OS.
    #[cfg(target_os = "android")]
    {
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            // First pass: look for our app's package name
            for line in maps.lines() {
                if let Some(path_str) = line.split_whitespace().last() {
                    if path_str.contains("com.lovitus.zashboard") && path_str.ends_with(".so") {
                        let path = PathBuf::from(path_str);
                        if let Some(parent) = path.parent() {
                            eprintln!("Android native lib dir (by package): {}", parent.display());
                            return parent.to_path_buf();
                        }
                    }
                }
            }
            // Second pass: any non-system .so with /lib/ in path
            for line in maps.lines() {
                if let Some(path_str) = line.split_whitespace().last() {
                    if path_str.ends_with(".so")
                        && path_str.contains("/lib/")
                        && !path_str.starts_with("/system/")
                        && !path_str.starts_with("/vendor/")
                        && !path_str.starts_with("/apex/")
                    {
                        let path = PathBuf::from(path_str);
                        if let Some(parent) = path.parent() {
                            eprintln!("Android native lib dir (fallback): {}", parent.display());
                            return parent.to_path_buf();
                        }
                    }
                }
            }
        }
        eprintln!("WARNING: Could not detect Android native lib dir from /proc/self/maps");
    }

    // Desktop: use cache dir when embedded binaries are available
    #[cfg(not(target_os = "android"))]
    if !GUST_GZ.is_empty() || !SLIDER_GZ.is_empty() {
        return get_sidecar_cache_dir();
    }

    // Dev mode fallback: next to executable
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
    // Ensure binary is available (extract from embedded if needed)
    ensure_sidecar_available(sidecar_dir, tool)?;

    let tool_path = resolve_tool_path(sidecar_dir, tool);

    // Pre-launch diagnostics
    if !tool_path.exists() {
        return Err(format!(
            "Binary not found: {}. Sidecar dir contents: {:?}",
            tool_path.display(),
            std::fs::read_dir(sidecar_dir)
                .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect::<Vec<_>>())
                .unwrap_or_default()
        ));
    }

    match std::fs::metadata(&tool_path) {
        Ok(meta) => {
            if meta.len() == 0 {
                return Err(format!("Binary is empty (0 bytes): {}", tool_path.display()));
            }
            eprintln!("Binary: {} (size: {} bytes)", tool_path.display(), meta.len());

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = meta.permissions().mode();
                if mode & 0o111 == 0 {
                    eprintln!("Binary not executable (mode: {:o}), fixing...", mode);
                    std::fs::set_permissions(&tool_path, std::fs::Permissions::from_mode(0o755))
                        .map_err(|e| format!("Cannot set +x on {}: {}", tool_path.display(), e))?;
                }
            }
        }
        Err(e) => {
            return Err(format!("Cannot stat {}: {}", tool_path.display(), e));
        }
    }

    eprintln!("Starting tunnel: {} {:?}", tool_path.display(), args);

    let mut cmd = Command::new(&tool_path);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let mut child = cmd.spawn()
        .map_err(|e| {
            let msg = format!("Failed to start {}: {}", tool_path.display(), e);
            #[cfg(unix)]
            {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return format!("{} (Permission denied — SELinux or filesystem restriction?)", msg);
                }
            }
            msg
        })?;

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
    save_configs(&s.configs, &s.config_path);
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
        save_configs(&s.configs, &s.config_path);
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
        s.running_set.insert(id.clone());
    }

    // Wait briefly then check if process crashed immediately
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        if let Some(proc) = s.processes.get_mut(&id) {
            match proc.child.try_wait() {
                Ok(Some(exit_status)) => {
                    let logs: Vec<String> = proc.logs.lock().unwrap().iter().cloned().collect();
                    let error_msg = format!(
                        "Process exited immediately ({}). Output: {}",
                        exit_status,
                        if logs.is_empty() { "(no output captured)".to_string() } else { logs.join(" | ") }
                    );
                    eprintln!("Tunnel {} early exit: {}", id, error_msg);
                    s.processes.remove(&id);
                    s.running_set.remove(&id);
                    return Err(error_msg);
                }
                Ok(None) => {
                    eprintln!("Tunnel {} is running (pid: {:?})", id, pid);
                }
                Err(e) => {
                    eprintln!("WARNING: try_wait error for tunnel {}: {}", id, e);
                }
            }
        }
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
        s.running_set.remove(&id);
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
async fn health_check_tunnels(
    state: State<'_, Mutex<TunnelState>>,
) -> Result<Vec<String>, String> {
    // Collect dead tunnels that should still be running
    let to_restart: Vec<(String, String, Vec<String>, PathBuf)>;
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        let running_ids: Vec<String> = s.running_set.iter().cloned().collect();
        let mut dead_ids = Vec::new();

        for id in &running_ids {
            if let Some(proc) = s.processes.get_mut(id) {
                match proc.child.try_wait() {
                    Ok(Some(_exit)) => {
                        dead_ids.push(id.clone());
                    }
                    Ok(None) => {} // still running
                    Err(_) => {
                        dead_ids.push(id.clone());
                    }
                }
            }
            // If not in processes at all but in running_set, it crashed and was already cleaned up
            else {
                dead_ids.push(id.clone());
            }
        }

        to_restart = dead_ids
            .iter()
            .filter_map(|id| {
                let config = s.configs.iter().find(|c| &c.id == id)?;
                Some((
                    config.id.clone(),
                    config.tool.clone(),
                    config.args.clone(),
                    s.sidecar_dir.clone(),
                ))
            })
            .collect();

        // Clean up dead processes
        for id in &dead_ids {
            s.processes.remove(id);
        }
    }

    if to_restart.is_empty() {
        return Ok(Vec::new());
    }

    let mut restarted = Vec::new();
    for (id, tool, args, sidecar_dir) in to_restart {
        eprintln!("Health check: restarting tunnel {}", id);
        match spawn_tunnel(&sidecar_dir, &tool, &args) {
            Ok(process) => {
                let mut s = state.lock().map_err(|e| e.to_string())?;
                s.processes.insert(id.clone(), process);
                restarted.push(id);
            }
            Err(e) => {
                eprintln!("Health check: failed to restart {}: {}", id, e);
                // Remove from running_set so we don't loop-retry
                let mut s = state.lock().map_err(|e| e.to_string())?;
                s.running_set.remove(&id);
            }
        }
    }

    if !restarted.is_empty() {
        eprintln!("Health check: restarted {} tunnels", restarted.len());
    }
    Ok(restarted)
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
        win.unminimize().ok();
        win.show().ok();
        win.set_focus().ok();
        restore_active_upstream_if_needed(handle, &win);
    } else {
        let created = WebviewWindowBuilder::new(
            handle,
            "main",
            WebviewUrl::App("index.html".into()),
        )
        .title("Zashboard - Mihomo Dashboard")
        .inner_size(1200.0, 800.0)
        .min_inner_size(400.0, 600.0)
        .build();

        if let Ok(win) = created {
            restore_active_upstream_if_needed(handle, &win);
        }
    }
}

#[cfg(desktop)]
fn active_upstream_url(handle: &tauri::AppHandle) -> Option<String> {
    let ui_state_ref = handle.state::<Mutex<ui_manager::UiManagerState>>();
    let s = ui_state_ref.lock().ok()?;
    let marker = s.base_dir.join("ui_active_version.txt");
    let persisted = std::fs::read_to_string(marker).ok().unwrap_or_default();
    let persisted = persisted.trim();
    if persisted.is_empty() || persisted.eq_ignore_ascii_case("builtin") {
        return None;
    }
    s.server_port.map(|p| format!("http://127.0.0.1:{}", p))
}

#[cfg(desktop)]
fn restore_active_upstream_if_needed(handle: &tauri::AppHandle, win: &tauri::WebviewWindow) {
    let Some(url) = active_upstream_url(handle) else {
        return;
    };

    let win = win.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        let js = format!(
            "if (window.location.href !== '{0}') {{ window.location.href = '{0}'; }}",
            url
        );
        let _ = win.eval(&js);
    });
}

// --- App entry ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    job_object::init();

    let sidecar_dir = resolve_sidecar_dir();

    // Try to extract/verify sidecar binaries
    for tool in &["gust", "slider"] {
        match ensure_sidecar_available(&sidecar_dir, tool) {
            Ok(()) => {
                let path = resolve_tool_path(&sidecar_dir, tool);
                eprintln!("Sidecar ready: {} -> {}", tool, path.display());
            }
            Err(e) => eprintln!("WARNING: Sidecar {} not available: {}", tool, e),
        }
    }

    let config_path = get_config_path();
    eprintln!("Initial config path: {}", config_path.display());
    eprintln!("Sidecar dir: {}", sidecar_dir.display());

    let ts = TunnelState {
        processes: HashMap::new(),
        configs: load_configs(&config_path),
        sidecar_dir: sidecar_dir.clone(),
        config_path,
        running_set: HashSet::new(),
    };

    let ui_state = ui_manager::init_state();

    let mut builder = tauri::Builder::default();

    // Single instance: desktop only (not available on mobile)
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_or_create_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(Mutex::new(ts))
        .manage(Mutex::new(ui_state))
        .invoke_handler(tauri::generate_handler![
            get_tunnels,
            get_tunnel_statuses,
            save_tunnel,
            remove_tunnel,
            start_tunnel,
            stop_tunnel,
            health_check_tunnels,
            add_defender_exclusion,
            ui_manager::ui_fetch_releases,
            ui_manager::ui_download_version,
            ui_manager::ui_activate_version,
            ui_manager::ui_deactivate,
            ui_manager::ui_get_info,
            ui_manager::ui_delete_version,
            ui_manager::ui_set_custom_urls,
        ])
        .setup(move |app| {
            ui_manager::set_app_handle(app.handle().clone());

            // --- Mobile: fix config path using Tauri's app data dir ---
            #[cfg(mobile)]
            {
                match app.path().app_data_dir() {
                    Ok(app_dir) => {
                        std::fs::create_dir_all(&app_dir).ok();

                        // Rebuild UI manager state using the writable app data directory on mobile.
                        // dirs::data_dir() may resolve to a read-only location on some Android devices.
                        let ui_base_dir = app_dir.join("ui_versions");
                        {
                            let ui_state = app.state::<Mutex<ui_manager::UiManagerState>>();
                            let mut ui = ui_state.lock().unwrap();
                            if let Some(ref shutdown) = ui.server_shutdown {
                                shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            *ui = ui_manager::init_state_with_base_dir(ui_base_dir.clone());
                        }
                        eprintln!("Mobile UI base dir: {}", ui_base_dir.display());

                        let mobile_config_path = app_dir.join("tunnels.json");
                        eprintln!("Mobile config path: {}", mobile_config_path.display());

                        let state = app.state::<Mutex<TunnelState>>();
                        let mut s = state.lock().unwrap();
                        // Reload configs from the correct mobile path
                        s.configs = load_configs(&mobile_config_path);
                        s.config_path = mobile_config_path;
                    }
                    Err(e) => {
                        eprintln!("WARNING: Cannot resolve app data dir: {}", e);
                    }
                }
            }

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
                                SHOULD_EXIT.store(true, Ordering::SeqCst);
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
                            s.running_set.insert(id.clone());
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
        .run(|_app_handle, _event| {
            #[cfg(desktop)]
            if let tauri::RunEvent::ExitRequested { api, .. } = &_event {
                // Only prevent exit for window-close events, not explicit Quit
                if !SHOULD_EXIT.load(Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
        });
}
