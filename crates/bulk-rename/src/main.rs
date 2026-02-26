use mpt_bulk_rename::config::BulkRenameConfig;
use std::fs;
use std::path::PathBuf;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_bulk_rename=info".into()),
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
    let config: BulkRenameConfig =
        mpt_common::config::load_module_config("bulk-rename").unwrap_or_default();

    // Collect files from CLI arguments (passed by file manager)
    let initial_files: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .flat_map(|p| {
            if p.is_dir() {
                // If a directory is passed, list its entries
                let list_opts = mpt_bulk_rename::ListOptions {
                    include_folders: config.include_folders,
                    include_subfolders: config.include_subfolders,
                };
                mpt_bulk_rename::Renamer::list_entries(&p, &list_opts).unwrap_or_default()
            } else if p.exists() {
                vec![p]
            } else {
                vec![]
            }
        })
        .collect();

    info!(
        "Bulk Rename GUI starting ({} initial files)",
        initial_files.len()
    );

    mpt_bulk_rename::gui::window::run_window(config, initial_files);
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-bulk-rename.lock")
}
