use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::rust_connection::RustConnection;

use crate::config::{CrosshairOrientation, MouseUtilsConfig};

/// Spawn the crosshairs overlay thread.
/// Creates thin line windows that follow the cursor.
pub fn spawn(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("crosshairs".into())
        .spawn(move || {
            if let Err(e) = run(config, stop) {
                warn!("Crosshairs stopped: {e}");
            }
        })
        .expect("failed to spawn crosshairs thread")
}

fn run(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> Result<()> {
    let (conn, screen_num) =
        RustConnection::connect(None).context("Crosshairs: failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let sw = screen.width_in_pixels;
    let sh = screen.height_in_pixels;
    let depth = screen.root_depth;

    let color = parse_hex_color(&config.crosshair_color);
    let thickness = config.crosshair_thickness.max(1);
    let _center_radius = config.crosshair_center_radius;
    let orientation = config.crosshair_orientation;
    let opacity_value = (config.crosshair_opacity.clamp(0.0, 1.0) * u32::MAX as f32) as u32;
    let fixed_length = if config.crosshair_fixed_length {
        Some(config.crosshair_fixed_length_px)
    } else {
        None
    };

    let opacity_atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_OPACITY")?
        .reply()?
        .atom;

    // Create horizontal line window (if enabled)
    let h_win = if orientation != CrosshairOrientation::Vertical {
        Some(create_line_window(
            &conn,
            root,
            depth,
            color,
            opacity_value,
            opacity_atom,
            0,
            0,
            sw,
            thickness as u16,
        )?)
    } else {
        None
    };

    // Create vertical line window (if enabled)
    let v_win = if orientation != CrosshairOrientation::Horizontal {
        Some(create_line_window(
            &conn,
            root,
            depth,
            color,
            opacity_value,
            opacity_atom,
            0,
            0,
            thickness as u16,
            sh,
        )?)
    } else {
        None
    };

    conn.flush()?;
    info!(
        "Crosshairs overlay active (orientation={:?}, thickness={}px)",
        orientation, thickness
    );

    let half_t = (thickness / 2) as i16;

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let reply = conn.query_pointer(root)?.reply()?;
        let cx = reply.root_x;
        let cy = reply.root_y;

        // Update horizontal line position
        if let Some(win) = h_win {
            let (hx, hw) = if let Some(len) = fixed_length {
                let half_len = (len / 2) as i16;
                ((cx - half_len).max(0), len.min(sw as u32) as u16)
            } else {
                (0i16, sw)
            };

            // Gap around cursor (center_radius)
            // We use two segments: left of gap and right of gap
            // But for simplicity with single window, just position it and accept the overlap
            let values = xproto::ConfigureWindowAux::new()
                .x(hx as i32)
                .y((cy - half_t) as i32)
                .width(hw as u32)
                .stack_mode(xproto::StackMode::ABOVE);
            conn.configure_window(win, &values)?;
        }

        // Update vertical line position
        if let Some(win) = v_win {
            let (vy, vh) = if let Some(len) = fixed_length {
                let half_len = (len / 2) as i16;
                ((cy - half_len).max(0), len.min(sh as u32) as u16)
            } else {
                (0i16, sh)
            };

            let values = xproto::ConfigureWindowAux::new()
                .x((cx - half_t) as i32)
                .y(vy as i32)
                .height(vh as u32)
                .stack_mode(xproto::StackMode::ABOVE);
            conn.configure_window(win, &values)?;
        }

        conn.flush()?;
        thread::sleep(Duration::from_millis(16)); // ~60fps
    }

    // Cleanup
    if let Some(win) = h_win {
        conn.destroy_window(win)?;
    }
    if let Some(win) = v_win {
        conn.destroy_window(win)?;
    }
    conn.flush()?;

    info!("Crosshairs overlay stopped");
    Ok(())
}

fn create_line_window(
    conn: &RustConnection,
    root: u32,
    depth: u8,
    color: u32,
    opacity_value: u32,
    opacity_atom: u32,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
) -> Result<u32> {
    let wid = conn.generate_id()?;

    let values = xproto::CreateWindowAux::new()
        .background_pixel(color)
        .override_redirect(1)
        .event_mask(xproto::EventMask::EXPOSURE);

    conn.create_window(
        depth,
        wid,
        root,
        x,
        y,
        w.max(1),
        h.max(1),
        0,
        xproto::WindowClass::INPUT_OUTPUT,
        0,
        &values,
    )?;

    // Set opacity
    conn.change_property(
        xproto::PropMode::REPLACE,
        wid,
        opacity_atom,
        xproto::AtomEnum::CARDINAL,
        32,
        1,
        &opacity_value.to_ne_bytes(),
    )?;

    // Make it click-through: use SHAPE input to make it have zero input region
    use x11rb::protocol::shape;
    let empty_pixmap = conn.generate_id()?;
    conn.create_pixmap(1, empty_pixmap, wid, 1, 1)?;
    let gc = conn.generate_id()?;
    conn.create_gc(gc, empty_pixmap, &xproto::CreateGCAux::new().foreground(0))?;
    conn.poly_fill_rectangle(
        empty_pixmap,
        gc,
        &[xproto::Rectangle {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }],
    )?;
    shape::mask(
        conn,
        shape::SO::SET,
        shape::SK::INPUT,
        wid,
        0,
        0,
        empty_pixmap,
    )?;
    conn.free_gc(gc)?;
    conn.free_pixmap(empty_pixmap)?;

    // Raise and map
    let raise = xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE);
    conn.configure_window(wid, &raise)?;
    conn.map_window(wid)?;

    Ok(wid)
}

fn parse_hex_color(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16).unwrap_or(0xFF0000)
}
