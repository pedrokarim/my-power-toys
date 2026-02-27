use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{debug, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

/// Spawn the cursor wrap thread.
/// When the cursor hits a screen edge, it wraps to the opposite side.
pub fn spawn(stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("cursor-wrap".into())
        .spawn(move || {
            if let Err(e) = run(stop) {
                warn!("Cursor Wrap stopped: {e}");
            }
        })
        .expect("failed to spawn cursor-wrap thread")
}

fn run(stop: Arc<AtomicBool>) -> Result<()> {
    let (conn, screen_num) =
        RustConnection::connect(None).context("Cursor Wrap: failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let sw = screen.width_in_pixels as i16;
    let sh = screen.height_in_pixels as i16;

    // Margin: how close to the edge before wrapping (in pixels)
    let margin: i16 = 1;

    info!("Cursor Wrap active (screen {}x{})", sw, sh);

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let reply = conn.query_pointer(root)?.reply()?;
        let cx = reply.root_x;
        let cy = reply.root_y;

        let mut nx = cx;
        let mut ny = cy;

        // Wrap horizontally
        if cx <= margin {
            nx = sw - margin - 2;
        } else if cx >= sw - margin - 1 {
            nx = margin + 1;
        }

        // Wrap vertically
        if cy <= margin {
            ny = sh - margin - 2;
        } else if cy >= sh - margin - 1 {
            ny = margin + 1;
        }

        if nx != cx || ny != cy {
            debug!("Cursor Wrap: ({},{}) -> ({},{})", cx, cy, nx, ny);
            conn.warp_pointer(0u32, root, 0, 0, 0, 0, nx, ny)?;
            conn.flush()?;
        }

        thread::sleep(Duration::from_millis(8)); // ~120Hz polling for responsive wrapping
    }

    info!("Cursor Wrap stopped");
    Ok(())
}
