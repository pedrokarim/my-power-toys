use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_shortcut_guide=info".into()),
        )
        .init();

    // Prevent multiple instances using atomic lock file creation.
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

    let config: mpt_shortcut_guide::config::ShortcutGuideConfig =
        mpt_common::config::load_module_config("shortcut-guide").unwrap_or_default();

    info!("Shortcut Guide GUI starting");
    mpt_shortcut_guide::gui::overlay::run_guide(config);
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-shortcut-guide.lock")
}
