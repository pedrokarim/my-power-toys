use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_workspaces=info".into()),
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

    let _ = fs::write(&lock_path, std::process::id().to_string());
    let lock_path_cleanup = lock_path.clone();
    let _guard = scopeguard::guard((), move |_| {
        let _ = fs::remove_file(&lock_path_cleanup);
    });

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--editor") {
        run_editor_mode();
    } else if args.iter().any(|a| a == "--launch") {
        run_launch_mode(&args);
    } else {
        error!("Usage: mpt-workspaces --editor");
        error!("       mpt-workspaces --launch <workspace-name>");
        std::process::exit(1);
    }
}

fn run_editor_mode() {
    info!("Workspaces editor starting");

    let config: mpt_workspaces::config::WorkspacesConfig =
        mpt_common::config::load_module_config("workspaces").unwrap_or_default();

    info!("Loaded {} workspace(s)", config.workspaces.len());
    mpt_workspaces::gui::window::run_editor(config);
}

fn run_launch_mode(args: &[String]) {
    let name = args
        .iter()
        .position(|a| a == "--launch")
        .and_then(|i| args.get(i + 1));

    let Some(name) = name else {
        error!("Missing workspace name after --launch");
        std::process::exit(1);
    };

    info!("Launching workspace: {name}");

    let mut config: mpt_workspaces::config::WorkspacesConfig =
        mpt_common::config::load_module_config("workspaces").unwrap_or_default();

    let ws = config.workspaces.iter_mut().find(|ws| ws.name == *name);

    let Some(ws) = ws else {
        error!("Workspace \"{name}\" not found");
        std::process::exit(1);
    };

    let statuses = mpt_workspaces::launcher::launch_workspace(ws);

    // Save updated last_launched
    if let Err(e) = mpt_common::config::save_module_config("workspaces", &config) {
        error!("Failed to save config: {e}");
    }

    for status in &statuses {
        match status {
            mpt_workspaces::launcher::LaunchStatus::Launched { app_name } => {
                info!("  Launched: {app_name}");
            }
            mpt_workspaces::launcher::LaunchStatus::Repositioned { app_name } => {
                info!("  Repositioned: {app_name}");
            }
            mpt_workspaces::launcher::LaunchStatus::Failed { app_name, error } => {
                error!("  Failed: {app_name}: {error}");
            }
            mpt_workspaces::launcher::LaunchStatus::Skipped { app_name } => {
                info!("  Skipped: {app_name}");
            }
        }
    }
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-workspaces.lock")
}
