use anyhow::{Result, bail};
use zbus::Connection;

const BUS_NAME: &str = "org.mypowertoys.Daemon";
const OBJECT_PATH: &str = "/org/mypowertoys/Daemon";
const INTERFACE: &str = "org.mypowertoys.Daemon";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = args[1].as_str();
    match command {
        "list" => cmd_list().await?,
        "start" => {
            let module = require_arg(&args, 2, "module id")?;
            cmd_start(&module).await?;
        }
        "stop" => {
            let module = require_arg(&args, 2, "module id")?;
            cmd_stop(&module).await?;
        }
        "trigger" => {
            let module = require_arg(&args, 2, "module id")?;
            cmd_trigger(&module).await?;
        }
        "ping" => cmd_ping().await?,
        "check-update" => cmd_check_update()?,
        "update" => cmd_update()?,
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-V" => {
            println!("mpt-ctl {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
            std::process::exit(1);
        }
    }
    Ok(())
}

fn require_arg(args: &[String], idx: usize, name: &str) -> Result<String> {
    args.get(idx)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing argument: {name}"))
}

fn print_usage() {
    println!(
        "mpt-ctl — MyPowerToys CLI\n\
         \n\
         USAGE:\n\
         \x20 mpt-ctl <command> [args]\n\
         \n\
         COMMANDS:\n\
         \x20 list              List all modules and their status\n\
         \x20 start <module>    Start a module\n\
         \x20 stop <module>     Stop a module\n\
         \x20 trigger <module>  Trigger a module's hotkey action\n\
         \x20 ping              Check if daemon is running\n\
         \x20 check-update      Check if a new version is available\n\
         \x20 update            Update to the latest version\n\
         \x20 help              Show this help\n\
         \x20 version           Show version"
    );
}

async fn connect() -> Result<Connection> {
    let conn = Connection::session().await?;
    Ok(conn)
}

async fn call_method(method: &str, args: Option<&str>) -> Result<zbus::Message> {
    let conn = connect().await?;
    let msg = if let Some(arg) = args {
        conn.call_method(Some(BUS_NAME), OBJECT_PATH, Some(INTERFACE), method, &arg)
            .await?
    } else {
        conn.call_method(Some(BUS_NAME), OBJECT_PATH, Some(INTERFACE), method, &())
            .await?
    };
    Ok(msg)
}

async fn cmd_list() -> Result<()> {
    let msg = call_method("ListModules", None)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to daemon. Is mpt-daemon running?\n{e}"))?;

    let body: Vec<(String, String, bool)> = msg.body().deserialize()?;

    if body.is_empty() {
        println!("No modules registered.");
        return Ok(());
    }

    println!("{:<20} {:<25} STATUS", "ID", "NAME");
    println!("{}", "-".repeat(55));
    for (id, name, running) in &body {
        let status = if *running { "running" } else { "stopped" };
        println!("{id:<20} {name:<25} {status}");
    }
    println!("\n{} module(s) total", body.len());
    Ok(())
}

async fn cmd_start(module: &str) -> Result<()> {
    let msg = call_method("StartModule", Some(module)).await?;
    let result: String = msg.body().deserialize()?;
    if result == "ok" {
        println!("Started module: {module}");
    } else {
        bail!("Failed to start {module}: {result}");
    }
    Ok(())
}

async fn cmd_stop(module: &str) -> Result<()> {
    let msg = call_method("StopModule", Some(module)).await?;
    let result: String = msg.body().deserialize()?;
    if result == "ok" {
        println!("Stopped module: {module}");
    } else {
        bail!("Failed to stop {module}: {result}");
    }
    Ok(())
}

async fn cmd_trigger(module: &str) -> Result<()> {
    let msg = call_method("TriggerHotkey", Some(module)).await?;
    let result: String = msg.body().deserialize()?;
    if result == "ok" {
        println!("Triggered hotkey for: {module}");
    } else {
        bail!("Failed to trigger {module}: {result}");
    }
    Ok(())
}

async fn cmd_ping() -> Result<()> {
    let msg = call_method("Ping", None)
        .await
        .map_err(|_| anyhow::anyhow!("Daemon is not running."))?;
    let result: String = msg.body().deserialize()?;
    println!("Daemon response: {result}");
    Ok(())
}

fn cmd_check_update() -> Result<()> {
    println!("Checking for updates...");
    match mpt_common::updater::check_for_update() {
        Ok(Some(info)) => {
            println!("Update available: {} -> {}", info.current_version, info.latest_version);
            if !info.release_notes.is_empty() {
                println!("\nRelease notes:\n{}", info.release_notes);
            }
            println!("\nRun `mpt-ctl update` to install.");
        }
        Ok(None) => {
            println!("Already up to date (v{}).", env!("CARGO_PKG_VERSION"));
        }
        Err(e) => {
            bail!("Failed to check for updates: {e}");
        }
    }
    Ok(())
}

fn cmd_update() -> Result<()> {
    println!("Checking for updates...");
    match mpt_common::updater::check_for_update() {
        Ok(Some(info)) => {
            println!(
                "Updating {} -> {} ...",
                info.current_version, info.latest_version
            );
            let version = mpt_common::updater::perform_update("mpt-ctl")?;
            println!("Successfully updated to v{version}.");
            println!("Restart the daemon to complete the update: systemctl --user restart mpt-daemon");
        }
        Ok(None) => {
            println!("Already up to date (v{}).", env!("CARGO_PKG_VERSION"));
        }
        Err(e) => {
            bail!("Failed to check for updates: {e}");
        }
    }
    Ok(())
}
