use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_quick_accent=info".into()),
        )
        .init();

    // Prevent multiple overlays
    let lock_path = lock_file_path();
    if lock_path.exists() {
        if let Ok(pid_str) = fs::read_to_string(&lock_path)
            && let Ok(pid) = pid_str.trim().parse::<u32>()
            && std::path::Path::new(&format!("/proc/{pid}")).exists()
        {
            info!("Quick Accent overlay already running (pid={pid}), exiting");
            std::process::exit(0);
        }
        let _ = fs::remove_file(&lock_path);
    }

    let _ = fs::write(&lock_path, std::process::id().to_string());
    let lock_path_cleanup = lock_path.clone();
    let _guard = scopeguard::guard((), move |_| {
        let _ = fs::remove_file(&lock_path_cleanup);
    });

    let args: Vec<String> = std::env::args().collect();

    let chars_str = parse_arg(&args, "--chars").unwrap_or_default();
    let accents: Vec<char> = chars_str.chars().collect();
    let backspaces: u32 = parse_arg(&args, "--backspaces")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let position = parse_arg(&args, "--position").unwrap_or_else(|| "above-cursor".into());

    if accents.is_empty() {
        error!("No accent characters provided");
        std::process::exit(1);
    }

    info!(
        "Quick Accent overlay: accents={chars_str}, backspaces={backspaces}, position={position}"
    );

    let selected = mpt_quick_accent::overlay::run_overlay(&accents, &position);

    match selected {
        Some(ch) => {
            info!("Selected: {ch}");

            // Small delay to let focus return to the original window
            thread::sleep(Duration::from_millis(80));

            // Send backspaces to erase the typed characters
            if backspaces > 0 {
                let bs_keys: Vec<&str> = (0..backspaces).map(|_| "BackSpace").collect();
                let bs_arg = bs_keys.join(" ");
                let _ = Command::new("xdotool")
                    .args(["key", "--clearmodifiers", &bs_arg])
                    .status();

                // Small delay between backspaces and typing
                thread::sleep(Duration::from_millis(30));
            }

            // Type the accented character
            let _ = Command::new("xdotool")
                .args(["type", "--clearmodifiers", &ch.to_string()])
                .status();
        }
        None => {
            info!("Quick Accent cancelled");
        }
    }
}

fn parse_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-quick-accent.lock")
}
