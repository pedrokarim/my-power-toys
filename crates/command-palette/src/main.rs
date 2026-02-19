use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_command_palette=info".into()),
        )
        .init();

    // Prevent multiple instances using atomic lock file creation
    let lock_path = lock_file_path();

    // Check if an existing instance is still alive
    if lock_path.exists() {
        if let Ok(pid_str) = fs::read_to_string(&lock_path)
            && let Ok(pid) = pid_str.trim().parse::<u32>()
            && std::path::Path::new(&format!("/proc/{pid}")).exists()
        {
            info!("Another instance is already running (pid={pid}), exiting");
            std::process::exit(0);
        }
        // Stale lock file, remove it
        let _ = fs::remove_file(&lock_path);
    }

    // Atomically create the lock file (fails if another process beat us to it)
    let lock_result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path);

    match lock_result {
        Ok(mut file) => {
            let _ = file.write_all(std::process::id().to_string().as_bytes());
        }
        Err(_) => {
            // Another process created the lock between our check and create
            info!("Lock file contention, another instance likely starting, exiting");
            std::process::exit(0);
        }
    }

    // Ensure lock file is cleaned up on exit
    let lock_path_cleanup = lock_path.clone();
    let _guard = scopeguard::guard((), move |_| {
        let _ = fs::remove_file(&lock_path_cleanup);
    });

    // Load config
    let config: mpt_command_palette::CommandPaletteConfig =
        mpt_common::config::load_module_config("command-palette").unwrap_or_default();

    info!("Command Palette GUI starting");

    mpt_command_palette::gui::overlay::run_palette(config);
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-command-palette.lock")
}
