use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::protocol::xinput::{self, ConnectionExt as XiConnectionExt};
use x11rb::rust_connection::RustConnection;

use crate::config::MouseUtilsConfig;

/// Spawn the mouse highlighter thread.
/// Watches for mouse clicks and shows colored circles at click positions.
pub fn spawn(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("mouse-highlighter".into())
        .spawn(move || {
            if let Err(e) = run(config, stop) {
                warn!("Mouse Highlighter stopped: {e}");
            }
        })
        .expect("failed to spawn mouse-highlighter thread")
}

/// A circle overlay that fades out after a delay.
struct HighlightCircle {
    window: u32,
    created_at: Instant,
}

fn run(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> Result<()> {
    let (conn, screen_num) =
        RustConnection::connect(None).context("Highlighter: failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let depth = screen.root_depth;

    // Select XI2 raw button events
    // XI_RawButtonPress = bit 15, XI_RawButtonRelease = bit 16
    let mask: u32 = (1 << 15) | (1 << 16);
    let event_mask = xinput::EventMask {
        deviceid: 3,
        mask: vec![mask.into()],
    };
    conn.xinput_xi_select_events(root, &[event_mask])?
        .check()
        .context("Highlighter: failed to select XInput2 events")?;
    conn.flush()?;

    let opacity_atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_OPACITY")?
        .reply()?
        .atom;

    let primary_color = parse_hex_color(&config.highlighter_primary_color);
    let secondary_color = parse_hex_color(&config.highlighter_secondary_color);
    let radius = config.highlighter_radius.max(5);
    let fade_delay = Duration::from_millis(config.highlighter_fade_delay_ms as u64);
    let fade_duration = Duration::from_millis(config.highlighter_fade_duration_ms as u64);
    let total_lifetime = fade_delay + fade_duration;

    info!(
        "Mouse Highlighter active (radius={}px, fade={}+{}ms)",
        radius, config.highlighter_fade_delay_ms, config.highlighter_fade_duration_ms
    );

    let mut circles: Vec<HighlightCircle> = Vec::new();

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        // Clean up expired circles
        let now = Instant::now();
        circles.retain(|c| {
            if now.duration_since(c.created_at) > total_lifetime {
                let _ = conn.destroy_window(c.window);
                false
            } else {
                true
            }
        });

        // Update opacity for fading circles
        for circle in &circles {
            let elapsed = now.duration_since(circle.created_at);
            if elapsed > fade_delay {
                let fade_progress = (elapsed - fade_delay).as_secs_f32() / fade_duration.as_secs_f32();
                let alpha = (1.0 - fade_progress).clamp(0.0, 1.0);
                let opacity_value = (alpha * u32::MAX as f32) as u32;
                let _ = conn.change_property(
                    xproto::PropMode::REPLACE,
                    circle.window,
                    opacity_atom,
                    xproto::AtomEnum::CARDINAL,
                    32,
                    1,
                    &opacity_value.to_ne_bytes(),
                );
            }
        }

        // Process events
        let event = conn.poll_for_event()?;
        if let Some(event) = event {
            if let x11rb::protocol::Event::XinputRawButtonPress(e) = &event {
                let button = e.detail;
                // button 1 = primary, button 3 = secondary
                // Ignore scroll buttons (4, 5, 6, 7)
                if button == 1 || button == 3 {
                    let color = if button == 1 {
                        primary_color
                    } else {
                        secondary_color
                    };

                    let cursor = conn.query_pointer(root)?.reply()?;
                    let cx = cursor.root_x;
                    let cy = cursor.root_y;

                    debug!("Highlighter: click at ({}, {}), button={}", cx, cy, button);

                    if let Ok(win) = create_circle_window(
                        &conn, root, depth, color, opacity_atom,
                        cx, cy, radius,
                    ) {
                        circles.push(HighlightCircle {
                            window: win,
                            created_at: Instant::now(),
                        });
                    }
                }
            }
        }

        let _ = conn.flush();
        thread::sleep(Duration::from_millis(30));
    }

    // Cleanup all circles
    for circle in &circles {
        let _ = conn.destroy_window(circle.window);
    }
    let _ = conn.flush();

    info!("Mouse Highlighter stopped");
    Ok(())
}

fn create_circle_window(
    conn: &RustConnection,
    root: u32,
    depth: u8,
    color: u32,
    opacity_atom: u32,
    cx: i16,
    cy: i16,
    radius: u32,
) -> Result<u32> {
    let r = radius as i16;
    let diameter = (radius * 2) as u16;

    let wid = conn.generate_id()?;
    let values = xproto::CreateWindowAux::new()
        .background_pixel(color)
        .override_redirect(1)
        .event_mask(xproto::EventMask::EXPOSURE);

    conn.create_window(
        depth,
        wid,
        root,
        cx - r,
        cy - r,
        diameter,
        diameter,
        0,
        xproto::WindowClass::INPUT_OUTPUT,
        0,
        &values,
    )?;

    // Apply circular SHAPE
    use x11rb::protocol::shape;

    let pixmap = conn.generate_id()?;
    conn.create_pixmap(1, pixmap, wid, diameter, diameter)?;

    let gc_black = conn.generate_id()?;
    conn.create_gc(gc_black, pixmap, &xproto::CreateGCAux::new().foreground(0))?;
    let gc_white = conn.generate_id()?;
    conn.create_gc(gc_white, pixmap, &xproto::CreateGCAux::new().foreground(1))?;

    // Fill black (transparent)
    conn.poly_fill_rectangle(
        pixmap,
        gc_black,
        &[xproto::Rectangle {
            x: 0,
            y: 0,
            width: diameter,
            height: diameter,
        }],
    )?;

    // Draw white circle (opaque)
    conn.poly_fill_arc(
        pixmap,
        gc_white,
        &[xproto::Arc {
            x: 0,
            y: 0,
            width: diameter,
            height: diameter,
            angle1: 0,
            angle2: 360 * 64,
        }],
    )?;

    // Apply shape (bounding = circle shape, input = empty so clicks pass through)
    shape::mask(conn, shape::SO::SET, shape::SK::BOUNDING, wid, 0, 0, pixmap)?;

    // Make click-through
    let empty_pix = conn.generate_id()?;
    conn.create_pixmap(1, empty_pix, wid, 1, 1)?;
    let gc_empty = conn.generate_id()?;
    conn.create_gc(gc_empty, empty_pix, &xproto::CreateGCAux::new().foreground(0))?;
    conn.poly_fill_rectangle(
        empty_pix,
        gc_empty,
        &[xproto::Rectangle { x: 0, y: 0, width: 1, height: 1 }],
    )?;
    shape::mask(conn, shape::SO::SET, shape::SK::INPUT, wid, 0, 0, empty_pix)?;

    conn.free_gc(gc_black)?;
    conn.free_gc(gc_white)?;
    conn.free_gc(gc_empty)?;
    conn.free_pixmap(pixmap)?;
    conn.free_pixmap(empty_pix)?;

    // Set full opacity initially
    let full_opacity = u32::MAX;
    conn.change_property(
        xproto::PropMode::REPLACE,
        wid,
        opacity_atom,
        xproto::AtomEnum::CARDINAL,
        32,
        1,
        &full_opacity.to_ne_bytes(),
    )?;

    // Raise and map
    let raise = xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE);
    conn.configure_window(wid, &raise)?;
    conn.map_window(wid)?;

    Ok(wid)
}

fn parse_hex_color(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16).unwrap_or(0xFFFF00)
}
