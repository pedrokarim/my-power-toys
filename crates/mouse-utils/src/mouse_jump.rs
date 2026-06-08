use crate::config::MouseUtilsConfig;
use anyhow::{Context, Result};
use mpt_common::hotkey::{Hotkey, Modifier};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{self, ConnectionExt, GrabMode, ModMask};
use x11rb::rust_connection::RustConnection;

// ── X11 modifier masks ────────────────────────────────────────────────────

const MOD_SHIFT: u16 = 1 << 0;
const MOD_LOCK: u16 = 1 << 1;
const MOD_CONTROL: u16 = 1 << 2;
const MOD_ALT: u16 = 1 << 3;
const MOD_NUMLOCK: u16 = 1 << 4;
const MOD_SUPER: u16 = 1 << 6;

// ── Monitor colors palette ────────────────────────────────────────────────

const MONITOR_COLORS: &[u32] = &[
    0x89B4FA, // blue
    0xA6E3A1, // green
    0xF9E2AF, // yellow
    0xFAB387, // orange
    0xCBA6F7, // purple
    0xF38BA8, // pink
];
const BG_COLOR: u32 = 0x1E1E2E;
const BORDER_COLOR: u32 = 0x45475A;
const PADDING: i32 = 12;
const BORDER_WIDTH: i32 = 2;

// ── Public API ────────────────────────────────────────────────────────────

pub fn spawn(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("mouse-jump".into())
        .spawn(move || {
            if let Err(e) = run(&config, &stop) {
                warn!("Mouse Jump stopped: {e}");
            }
        })
        .expect("failed to spawn mouse-jump thread")
}

// ── Main loop ─────────────────────────────────────────────────────────────

fn run(config: &MouseUtilsConfig, stop: &AtomicBool) -> Result<()> {
    let shortcut_str = if config.mouse_jump_shortcut.is_empty() {
        "Super+Shift+D"
    } else {
        &config.mouse_jump_shortcut
    };

    let parsed = Hotkey::parse(shortcut_str)
        .with_context(|| format!("invalid mouse jump shortcut: {shortcut_str}"))?;

    let (conn, screen_num) =
        RustConnection::connect(None).context("Mouse Jump: failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Grab the shortcut key on the root window
    let modifiers = modifiers_to_x11(&parsed.modifiers);
    let keysyms = hotkey_key_to_keysyms(&parsed.key);
    if keysyms.is_empty() {
        anyhow::bail!("Mouse Jump: unknown key '{}'", parsed.key);
    }
    let keycodes = keycodes_for_keysyms(&conn, &keysyms)?;
    if keycodes.is_empty() {
        anyhow::bail!("Mouse Jump: no keycode found for '{}'", parsed.key);
    }

    for &kc in &keycodes {
        if let Err(e) = grab_with_lock_variants(&conn, root, kc, modifiers) {
            warn!("Mouse Jump: could not grab keycode {kc}: {e}");
        }
    }
    conn.flush()?;

    info!(
        "Mouse Jump active (shortcut={}, keycodes={:?})",
        shortcut_str, keycodes
    );

    // Event loop
    while !stop.load(Ordering::Relaxed) {
        if let Some(event) = conn.poll_for_event()? {
            if let Event::KeyPress(ev) = event {
                let ev_mod = normalize_mod_state(ev.state.into());
                if keycodes.contains(&ev.detail) && ev_mod == modifiers {
                    debug!("Mouse Jump: shortcut triggered");
                    if let Err(e) = show_popup(&conn, root, config) {
                        warn!("Mouse Jump popup error: {e}");
                    }
                }
            }
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }

    // Ungrab
    for &kc in &keycodes {
        let _ = ungrab_with_lock_variants(&conn, root, kc, modifiers);
    }
    conn.flush()?;
    info!("Mouse Jump stopped");
    Ok(())
}

// ── Popup ─────────────────────────────────────────────────────────────────

fn show_popup(conn: &RustConnection, root: u32, config: &MouseUtilsConfig) -> Result<()> {
    let monitors =
        mpt_common::monitor::detect_monitors().context("Mouse Jump: cannot detect monitors")?;
    if monitors.is_empty() {
        anyhow::bail!("no monitors detected");
    }

    // Compute bounding box of the virtual desktop
    let min_x = monitors.iter().map(|m| m.x).min().unwrap();
    let min_y = monitors.iter().map(|m| m.y).min().unwrap();
    let max_x = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap();
    let max_y = monitors
        .iter()
        .map(|m| m.y + m.height as i32)
        .max()
        .unwrap();
    let total_w = (max_x - min_x) as f64;
    let total_h = (max_y - min_y) as f64;

    if total_w <= 0.0 || total_h <= 0.0 {
        anyhow::bail!("invalid desktop dimensions");
    }

    // Scale to fit within max_width x max_height
    let max_w = config.mouse_jump_max_width as f64;
    let max_h = config.mouse_jump_max_height as f64;
    let scale = (max_w / total_w).min(max_h / total_h);

    let popup_w = (total_w * scale) as u16 + (PADDING * 2) as u16;
    let popup_h = (total_h * scale) as u16 + (PADDING * 2) as u16;

    // Find which monitor the cursor is on, center popup there
    let pointer = conn.query_pointer(root)?.reply()?;
    let cursor_x = pointer.root_x as i32;
    let cursor_y = pointer.root_y as i32;

    let current_monitor = monitors
        .iter()
        .find(|m| {
            cursor_x >= m.x
                && cursor_x < m.x + m.width as i32
                && cursor_y >= m.y
                && cursor_y < m.y + m.height as i32
        })
        .unwrap_or(&monitors[0]);

    // Center popup on cursor, but clamp to stay within the current monitor
    let popup_x = (cursor_x - popup_w as i32 / 2)
        .max(current_monitor.x)
        .min(current_monitor.x + current_monitor.width as i32 - popup_w as i32);
    let popup_y = (cursor_y - popup_h as i32 / 2)
        .max(current_monitor.y)
        .min(current_monitor.y + current_monitor.height as i32 - popup_h as i32);

    // Create the popup window
    let window = conn.generate_id()?;
    let values = xproto::CreateWindowAux::new()
        .background_pixel(BG_COLOR)
        .override_redirect(1)
        .event_mask(
            xproto::EventMask::KEY_PRESS
                | xproto::EventMask::BUTTON_PRESS
                | xproto::EventMask::EXPOSURE,
        );

    let screen = &conn.setup().roots[0];
    conn.create_window(
        screen.root_depth,
        window,
        root,
        popup_x as i16,
        popup_y as i16,
        popup_w,
        popup_h,
        0,
        xproto::WindowClass::INPUT_OUTPUT,
        0,
        &values,
    )?;

    // Raise above everything
    let raise = xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE);
    conn.configure_window(window, &raise)?;
    conn.map_window(window)?;

    // Grab keyboard so we get Escape even if focus changes
    conn.grab_keyboard(
        false,
        window,
        x11rb::CURRENT_TIME,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?
    .reply()?;

    conn.flush()?;

    // Draw monitor rectangles
    draw_monitors(conn, window, &monitors, min_x, min_y, scale)?;

    // Set input focus
    conn.set_input_focus(
        xproto::InputFocus::POINTER_ROOT,
        window,
        x11rb::CURRENT_TIME,
    )?;
    conn.flush()?;

    // Wait for click or Escape
    let result = popup_event_loop(conn, window, &monitors, min_x, min_y, scale)?;

    // Destroy popup
    conn.ungrab_keyboard(x11rb::CURRENT_TIME)?;
    conn.destroy_window(window)?;
    conn.flush()?;

    // Warp cursor if clicked
    if let Some((real_x, real_y)) = result {
        debug!("Mouse Jump: warping to ({real_x}, {real_y})");
        conn.warp_pointer(0u32, root, 0, 0, 0, 0, real_x as i16, real_y as i16)?;
        conn.flush()?;
    }

    Ok(())
}

fn draw_monitors(
    conn: &RustConnection,
    window: u32,
    monitors: &[mpt_common::monitor::Monitor],
    origin_x: i32,
    origin_y: i32,
    scale: f64,
) -> Result<()> {
    // GC for borders
    let gc_border = conn.generate_id()?;
    conn.create_gc(
        gc_border,
        window,
        &xproto::CreateGCAux::new().foreground(BORDER_COLOR),
    )?;

    for (i, mon) in monitors.iter().enumerate() {
        let color = MONITOR_COLORS[i % MONITOR_COLORS.len()];

        let rx = PADDING + ((mon.x - origin_x) as f64 * scale) as i32;
        let ry = PADDING + ((mon.y - origin_y) as f64 * scale) as i32;
        let rw = (mon.width as f64 * scale) as i32;
        let rh = (mon.height as f64 * scale) as i32;

        // Draw border rectangle
        conn.poly_fill_rectangle(
            window,
            gc_border,
            &[xproto::Rectangle {
                x: rx as i16,
                y: ry as i16,
                width: rw as u16,
                height: rh as u16,
            }],
        )?;

        // Draw inner filled rectangle (inset by BORDER_WIDTH)
        let gc_fill = conn.generate_id()?;
        conn.create_gc(
            gc_fill,
            window,
            &xproto::CreateGCAux::new().foreground(color),
        )?;
        conn.poly_fill_rectangle(
            window,
            gc_fill,
            &[xproto::Rectangle {
                x: (rx + BORDER_WIDTH) as i16,
                y: (ry + BORDER_WIDTH) as i16,
                width: (rw - BORDER_WIDTH * 2).max(1) as u16,
                height: (rh - BORDER_WIDTH * 2).max(1) as u16,
            }],
        )?;

        // Draw monitor name
        let label = &mon.name;
        let text_x = rx + BORDER_WIDTH + 4;
        let text_y = ry + BORDER_WIDTH + 14;
        // Use a dark color for text on light backgrounds
        let gc_text = conn.generate_id()?;
        conn.create_gc(
            gc_text,
            window,
            &xproto::CreateGCAux::new().foreground(0x1E1E2E),
        )?;
        conn.image_text8(
            window,
            gc_text,
            text_x as i16,
            text_y as i16,
            label.as_bytes(),
        )?;

        conn.free_gc(gc_fill)?;
        conn.free_gc(gc_text)?;
    }

    conn.free_gc(gc_border)?;
    conn.flush()?;
    Ok(())
}

fn popup_event_loop(
    conn: &RustConnection,
    window: u32,
    monitors: &[mpt_common::monitor::Monitor],
    origin_x: i32,
    origin_y: i32,
    scale: f64,
) -> Result<Option<(i32, i32)>> {
    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::ButtonPress(ev) => {
                // Only handle clicks inside our window
                if ev.event != window {
                    return Ok(None);
                }
                let px = ev.event_x as i32;
                let py = ev.event_y as i32;

                // Convert popup coords to real desktop coords
                let real_x = ((px - PADDING) as f64 / scale) as i32 + origin_x;
                let real_y = ((py - PADDING) as f64 / scale) as i32 + origin_y;

                return Ok(Some((real_x, real_y)));
            }
            Event::KeyPress(ev) => {
                // Look up the keysym for this keycode
                let keysym = keycode_to_keysym(conn, ev.detail)?;
                if keysym == xkeysym::key::Escape {
                    return Ok(None);
                }
            }
            Event::Expose(_) => {
                // Redraw
                draw_monitors(conn, window, monitors, origin_x, origin_y, scale)?;
            }
            _ => {}
        }
    }
}

fn keycode_to_keysym(conn: &RustConnection, keycode: u8) -> Result<u32> {
    let reply = conn
        .get_keyboard_mapping(keycode, 1)?
        .reply()
        .context("get_keyboard_mapping for keysym lookup")?;
    Ok(reply.keysyms.first().copied().unwrap_or(0))
}

// ── X11 hotkey helpers (duplicated from daemon/hotkeys.rs) ────────────────

fn modifiers_to_x11(modifiers: &[Modifier]) -> u16 {
    let mut mask = 0u16;
    for modifier in modifiers {
        mask |= match modifier {
            Modifier::Shift => MOD_SHIFT,
            Modifier::Ctrl => MOD_CONTROL,
            Modifier::Alt => MOD_ALT,
            Modifier::Super => MOD_SUPER,
        };
    }
    mask
}

fn normalize_mod_state(state: u16) -> u16 {
    let keyboard_mods = state & 0x00ff;
    keyboard_mods & !(MOD_LOCK | MOD_NUMLOCK)
}

fn grab_with_lock_variants(
    conn: &RustConnection,
    root: u32,
    keycode: u8,
    modifiers: u16,
) -> Result<()> {
    let variants = [
        modifiers,
        modifiers | MOD_LOCK,
        modifiers | MOD_NUMLOCK,
        modifiers | MOD_LOCK | MOD_NUMLOCK,
    ];
    for state in variants {
        conn.grab_key(
            false,
            root,
            ModMask::from(state),
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .context("grab_key request failed")?
        .check()
        .context("grab_key rejected by X server")?;
    }
    Ok(())
}

fn ungrab_with_lock_variants(
    conn: &RustConnection,
    root: u32,
    keycode: u8,
    modifiers: u16,
) -> Result<()> {
    let variants = [
        modifiers,
        modifiers | MOD_LOCK,
        modifiers | MOD_NUMLOCK,
        modifiers | MOD_LOCK | MOD_NUMLOCK,
    ];
    for state in variants {
        conn.ungrab_key(keycode, root, ModMask::from(state))?;
    }
    Ok(())
}

fn keycodes_for_keysyms(conn: &RustConnection, target_keysyms: &[u32]) -> Result<Vec<u8>> {
    let setup = conn.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;
    let count = max.saturating_sub(min).saturating_add(1);

    let reply = conn
        .get_keyboard_mapping(min, count)
        .context("get_keyboard_mapping request failed")?
        .reply()
        .context("get_keyboard_mapping reply failed")?;

    if reply.keysyms_per_keycode == 0 {
        return Ok(Vec::new());
    }

    let per_keycode = reply.keysyms_per_keycode as usize;
    let mut keycodes = Vec::new();
    for (offset, keysyms) in reply.keysyms.chunks(per_keycode).enumerate() {
        if keysyms.iter().any(|keysym| target_keysyms.contains(keysym)) {
            keycodes.push(min.saturating_add(offset as u8));
        }
    }
    keycodes.sort_unstable();
    keycodes.dedup();
    Ok(keycodes)
}

fn hotkey_key_to_keysyms(key: &str) -> Vec<u32> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    if trimmed.len() == 1 {
        let ch = trimmed.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            return vec![
                ch.to_ascii_uppercase() as u32,
                ch.to_ascii_lowercase() as u32,
            ];
        }
        return vec![ch as u32];
    }

    match trimmed.to_lowercase().as_str() {
        "space" => vec![xkeysym::key::space, xkeysym::key::KP_Space],
        "tab" => vec![xkeysym::key::Tab, xkeysym::key::KP_Tab],
        "enter" | "return" => vec![xkeysym::key::Return, xkeysym::key::KP_Enter],
        "esc" | "escape" => vec![xkeysym::key::Escape],
        "backspace" => vec![xkeysym::key::BackSpace],
        "delete" | "del" => vec![xkeysym::key::Delete, xkeysym::key::KP_Delete],
        "left" => vec![xkeysym::key::Left],
        "right" => vec![xkeysym::key::Right],
        "up" => vec![xkeysym::key::Up],
        "down" => vec![xkeysym::key::Down],
        "f1" => vec![xkeysym::key::F1],
        "f2" => vec![xkeysym::key::F2],
        "f3" => vec![xkeysym::key::F3],
        "f4" => vec![xkeysym::key::F4],
        "f5" => vec![xkeysym::key::F5],
        "f6" => vec![xkeysym::key::F6],
        "f7" => vec![xkeysym::key::F7],
        "f8" => vec![xkeysym::key::F8],
        "f9" => vec![xkeysym::key::F9],
        "f10" => vec![xkeysym::key::F10],
        "f11" => vec![xkeysym::key::F11],
        "f12" => vec![xkeysym::key::F12],
        _ => vec![],
    }
}
