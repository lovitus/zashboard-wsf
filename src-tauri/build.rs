fn main() {
    // Ensure embed directory and placeholder files exist for include_bytes!
    // CI replaces these with real gzip-compressed sidecar binaries
    let embed_dir = std::path::Path::new("embed");
    if !embed_dir.exists() {
        std::fs::create_dir_all(embed_dir).ok();
    }
    for name in &["gust.gz", "slider.gz"] {
        let path = embed_dir.join(name);
        if !path.exists() {
            std::fs::write(&path, &[] as &[u8]).ok();
        }
    }

    tauri_build::build()
}
