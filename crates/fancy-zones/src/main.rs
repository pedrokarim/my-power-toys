use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_fancy_zones=info".into()),
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

    // Parse --window-id argument
    let window_id = parse_window_id();
    let Some(window_id) = window_id else {
        error!("Usage: mpt-fancy-zones --window-id <WINDOW_ID>");
        std::process::exit(1);
    };

    info!("FancyZones overlay starting for window {window_id}");

    // Load config
    let config: mpt_fancy_zones::config::FancyZonesConfig =
        mpt_common::config::load_module_config("fancy-zones").unwrap_or_default();

    let layout = config
        .layouts
        .get(config.active_layout)
        .cloned()
        .unwrap_or_else(|| mpt_fancy_zones::layout::Layout::default_columns(2));

    let zone_gap = config.zone_gap;
    let layout_for_snap = layout.clone();

    // Show overlay and get selected zone
    let selected = mpt_fancy_zones::overlay::run_overlay(layout, zone_gap);

    match selected {
        Some(zone_idx) => {
            info!("Zone {zone_idx} selected, snapping window {window_id}");

            if let Some(zone) = layout_for_snap.zones.get(zone_idx) {
                match mpt_fancy_zones::x11::get_screen_geometry() {
                    Ok((sw, sh)) => {
                        let (x, y, w, h) = zone.to_pixels_with_gap(sw, sh, zone_gap);
                        if let Err(e) =
                            mpt_fancy_zones::x11::move_resize_window(window_id, x, y, w, h)
                        {
                            error!("Failed to snap window: {e}");
                        }
                    }
                    Err(e) => {
                        error!("Failed to get screen geometry: {e}");
                    }
                }
            }
        }
        None => {
            info!("Zone selection cancelled");
        }
    }
}

fn parse_window_id() -> Option<u32> {
    let args: Vec<String> = std::env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--window-id" {
            return args.get(i + 1)?.parse().ok();
        }
    }
    None
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-fancy-zones.lock")
}
