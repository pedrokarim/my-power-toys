use mpt_image_resizer::ImageResizerConfig;
use std::fs;
use std::path::PathBuf;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_image_resizer=info".into()),
        )
        .init();

    // Prevent multiple instances
    let lock_path = lock_file_path();
    if lock_path.exists() {
        if let Ok(pid_str) = fs::read_to_string(&lock_path)
            && let Ok(pid) = pid_str.trim().parse::<u32>()
        {
            let proc_path = format!("/proc/{pid}");
            if std::path::Path::new(&proc_path).exists() {
                info!("Another instance is already running (pid={pid}), exiting");
                std::process::exit(0);
            }
        }
        let _ = fs::remove_file(&lock_path);
    }

    // Write our PID
    let _ = fs::write(&lock_path, std::process::id().to_string());

    // Ensure lock file is cleaned up on exit
    let lock_path_cleanup = lock_path.clone();
    let _guard = scopeguard::guard((), move |_| {
        let _ = fs::remove_file(&lock_path_cleanup);
    });

    // Load config
    let config: ImageResizerConfig =
        mpt_common::config::load_module_config("image-resizer").unwrap_or_default();

    info!(
        "Image Resizer GUI starting (preset={:?}, format={:?}, quality={})",
        config.preset, config.output_format, config.quality
    );

    mpt_image_resizer::gui::window::run_window(config);
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-image-resizer.lock")
}
