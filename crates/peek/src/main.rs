use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_peek=info".into()),
        )
        .init();

    // Prevent multiple instances using atomic lock file creation
    let lock_path = lock_file_path();

    if lock_path.exists() {
        if let Ok(pid_str) = fs::read_to_string(&lock_path)
            && let Ok(pid) = pid_str.trim().parse::<u32>()
            && std::path::Path::new(&format!("/proc/{pid}")).exists()
        {
            info!("Another instance is already running (pid={pid}), exiting");
            std::process::exit(0);
        }
        let _ = fs::remove_file(&lock_path);
    }

    let lock_result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path);

    match lock_result {
        Ok(mut file) => {
            let _ = file.write_all(std::process::id().to_string().as_bytes());
        }
        Err(_) => {
            info!("Lock file contention, another instance likely starting, exiting");
            std::process::exit(0);
        }
    }

    let lock_path_cleanup = lock_path.clone();
    let _guard = scopeguard::guard((), move |_| {
        let _ = fs::remove_file(&lock_path_cleanup);
    });

    // Load config
    let config: mpt_peek::PeekConfig =
        mpt_common::config::load_module_config("peek").unwrap_or_default();

    // Determine file path: CLI argument or detect from clipboard
    let file_path = if let Some(arg) = std::env::args().nth(1) {
        let path = PathBuf::from(&arg);
        if path.exists() {
            Some(path)
        } else {
            error!("File not found: {arg}");
            None
        }
    } else {
        info!("No file argument, detecting from clipboard/file manager");
        mpt_peek::file_detect::detect_selected_file()
    };

    let file_path = match file_path {
        Some(p) => p,
        None => {
            error!("No file to preview. Usage: mpt-peek [filepath]");
            error!("Or copy a file in your file manager before pressing the hotkey.");
            std::process::exit(1);
        }
    };

    info!("Previewing: {}", file_path.display());

    // Generate preview
    let preview = match mpt_peek::preview::generate_preview(
        &file_path,
        config.max_preview_lines,
        config.max_dir_entries,
    ) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to generate preview: {e}");
            std::process::exit(1);
        }
    };

    info!(
        "Preview generated: kind={}, size={}",
        preview.kind,
        mpt_peek::preview::format_size(preview.size_bytes)
    );

    // Run GUI
    mpt_peek::gui::window::run_peek(preview, config);
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-peek.lock")
}
