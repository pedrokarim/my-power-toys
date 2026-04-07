use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xinput::{self, ConnectionExt as XiConnectionExt};
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::rust_connection::RustConnection;

use crate::config::{FindMyMouseActivation, MouseUtilsConfig};

/// Spawn the Find My Mouse monitor thread.
/// Watches for the configured activation (double Ctrl, shake, etc.) and shows a spotlight overlay.
pub fn spawn(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("find-my-mouse".into())
        .spawn(move || {
            if let Err(e) = run(config, stop) {
                warn!("Find My Mouse stopped: {e}");
            }
        })
        .expect("failed to spawn find-my-mouse thread")
}

fn run(config: MouseUtilsConfig, stop: Arc<AtomicBool>) -> Result<()> {
    let (conn, screen_num) =
        RustConnection::connect(None).context("Find My Mouse: failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Select XI2 raw key events on root window for double-Ctrl detection
    // XI_RawKeyPress = bit 13, XI_RawKeyRelease = bit 14
    // XI_RawButtonPress = bit 15 (for dismiss on click)
    let mask: u32 = (1 << 13) | (1 << 14) | (1 << 15);
    let event_mask = xinput::EventMask {
        deviceid: 3, // XIAllMasterDevices
        mask: vec![mask.into()],
    };
    conn.xinput_xi_select_events(root, &[event_mask])?
        .check()
        .context("Find My Mouse: failed to select XInput2 events")?;
    conn.flush()?;

    // Build keycode -> keysym map
    let keymap = build_keymap(&conn)?;

    info!(
        "Find My Mouse monitor active (activation={:?})",
        config.find_my_mouse_activation
    );

    let mut last_ctrl_time: Option<Instant> = None;
    let mut spotlight_active = false;
    let mut overlay: Option<SpotlightOverlay> = None;

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let event = conn.poll_for_event()?;
        let Some(event) = event else {
            // If spotlight is active, update cursor tracking
            if spotlight_active {
                if let Some(ref mut ov) = overlay {
                    ov.update_cursor()?;
                }
            }
            thread::sleep(Duration::from_millis(16)); // ~60fps when active, idle otherwise
            continue;
        };

        match &event {
            x11rb::protocol::Event::XinputRawKeyPress(e) => {
                let keycode = e.detail as u8;
                let keysym = keymap.get(&keycode).copied().unwrap_or(0);

                if spotlight_active {
                    // Any key dismisses the spotlight
                    debug!("Find My Mouse: dismissed by keypress");
                    spotlight_active = false;
                    if let Some(ov) = overlay.take() {
                        ov.destroy()?;
                    }
                    continue;
                }

                // Check for Ctrl press based on activation method
                match config.find_my_mouse_activation {
                    FindMyMouseActivation::LeftCtrlTwice
                    | FindMyMouseActivation::RightCtrlTwice => {
                        let is_target_ctrl = match config.find_my_mouse_activation {
                            FindMyMouseActivation::LeftCtrlTwice => {
                                keysym == xkeysym::key::Control_L
                            }
                            FindMyMouseActivation::RightCtrlTwice => {
                                keysym == xkeysym::key::Control_R
                            }
                            _ => false,
                        };

                        if is_target_ctrl {
                            let now = Instant::now();
                            if let Some(last) = last_ctrl_time {
                                if now.duration_since(last) < Duration::from_millis(400) {
                                    // Double press detected!
                                    debug!(
                                        "Find My Mouse: double Ctrl detected, activating spotlight"
                                    );
                                    last_ctrl_time = None;
                                    let ov = SpotlightOverlay::create(&config)?;
                                    overlay = Some(ov);
                                    spotlight_active = true;
                                    continue;
                                }
                            }
                            last_ctrl_time = Some(now);
                        }
                    }
                    _ => {}
                }
            }
            x11rb::protocol::Event::XinputRawButtonPress(_) => {
                if spotlight_active {
                    debug!("Find My Mouse: dismissed by click");
                    spotlight_active = false;
                    if let Some(ov) = overlay.take() {
                        ov.destroy()?;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(ov) = overlay.take() {
        let _ = ov.destroy();
    }

    info!("Find My Mouse monitor stopped");
    Ok(())
}

/// The spotlight overlay: a fullscreen dark window with a circular "hole" via SHAPE extension.
struct SpotlightOverlay {
    conn: RustConnection,
    window: u32,
    root: u32,
    screen_width: u16,
    screen_height: u16,
    radius: u32,
    _bg_color: u32,
    _spotlight_color: u32,
    _opacity: f32,
}

impl SpotlightOverlay {
    fn create(config: &MouseUtilsConfig) -> Result<Self> {
        let (conn, screen_num) =
            RustConnection::connect(None).context("spotlight: connect to X11")?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let screen_width = screen.width_in_pixels;
        let screen_height = screen.height_in_pixels;
        let depth = screen.root_depth;

        let bg_color = parse_hex_color(&config.find_my_mouse_bg_color);
        let spotlight_color = parse_hex_color(&config.find_my_mouse_spotlight_color);
        let radius = config.find_my_mouse_spotlight_radius;

        // Create fullscreen override-redirect window
        let window = conn.generate_id()?;
        let values = xproto::CreateWindowAux::new()
            .background_pixel(bg_color)
            .override_redirect(1)
            .event_mask(
                xproto::EventMask::EXPOSURE
                    | xproto::EventMask::KEY_PRESS
                    | xproto::EventMask::BUTTON_PRESS,
            );

        conn.create_window(
            depth,
            window,
            root,
            0,
            0,
            screen_width,
            screen_height,
            0,
            xproto::WindowClass::INPUT_OUTPUT,
            0,
            &values,
        )?;

        // Set opacity
        let opacity_value =
            (config.find_my_mouse_overlay_opacity.clamp(0.0, 1.0) * u32::MAX as f32) as u32;
        let opacity_atom = conn
            .intern_atom(false, b"_NET_WM_WINDOW_OPACITY")?
            .reply()?
            .atom;
        conn.change_property(
            xproto::PropMode::REPLACE,
            window,
            opacity_atom,
            xproto::AtomEnum::CARDINAL,
            32,
            1,
            &opacity_value.to_ne_bytes(),
        )?;

        // Raise above everything
        let raise = xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE);
        conn.configure_window(window, &raise)?;

        // Apply SHAPE to cut a circle around the cursor
        let cursor = get_cursor_position(&conn, root)?;
        apply_spotlight_shape(
            &conn,
            window,
            screen_width,
            screen_height,
            cursor.0,
            cursor.1,
            radius,
        )?;

        conn.map_window(window)?;
        conn.flush()?;

        info!(
            "Find My Mouse: spotlight overlay created at ({}, {})",
            cursor.0, cursor.1
        );

        Ok(Self {
            conn,
            window,
            root,
            screen_width,
            screen_height,
            radius,
            _bg_color: bg_color,
            _spotlight_color: spotlight_color,
            _opacity: config.find_my_mouse_overlay_opacity,
        })
    }

    fn update_cursor(&mut self) -> Result<()> {
        let (cx, cy) = get_cursor_position(&self.conn, self.root)?;
        apply_spotlight_shape(
            &self.conn,
            self.window,
            self.screen_width,
            self.screen_height,
            cx,
            cy,
            self.radius,
        )?;
        self.conn.flush()?;
        Ok(())
    }

    fn destroy(self) -> Result<()> {
        self.conn.destroy_window(self.window)?;
        self.conn.flush()?;
        debug!("Find My Mouse: spotlight overlay destroyed");
        Ok(())
    }
}

/// Apply a SHAPE bounding region that is the entire window minus a circle.
/// This creates the "spotlight" effect: dark everywhere except around the cursor.
fn apply_spotlight_shape(
    conn: &RustConnection,
    window: u32,
    sw: u16,
    sh: u16,
    cx: i16,
    cy: i16,
    radius: u32,
) -> Result<()> {
    use x11rb::protocol::shape;

    let r = radius as i16;

    // Create a pixmap for the shape mask (1-bit depth)
    let pixmap = conn.generate_id()?;
    conn.create_pixmap(1, pixmap, window, sw, sh)?;

    // Create a GC for drawing on the pixmap
    let gc_white = conn.generate_id()?;
    conn.create_gc(gc_white, pixmap, &xproto::CreateGCAux::new().foreground(1))?;
    let gc_black = conn.generate_id()?;
    conn.create_gc(gc_black, pixmap, &xproto::CreateGCAux::new().foreground(0))?;

    // Fill entire pixmap with white (opaque = window is visible)
    conn.poly_fill_rectangle(
        pixmap,
        gc_white,
        &[xproto::Rectangle {
            x: 0,
            y: 0,
            width: sw,
            height: sh,
        }],
    )?;

    // Draw black circle at cursor position (transparent = hole in the window)
    // We use poly_fill_arc to draw a filled circle
    conn.poly_fill_arc(
        pixmap,
        gc_black,
        &[xproto::Arc {
            x: cx - r,
            y: cy - r,
            width: (r * 2) as u16,
            height: (r * 2) as u16,
            angle1: 0,
            angle2: 360 * 64, // X11 uses 1/64th degree units
        }],
    )?;

    // Apply shape mask to the window
    shape::mask(
        conn,
        shape::SO::SET,
        shape::SK::BOUNDING,
        window,
        0,
        0,
        pixmap,
    )?;

    // Cleanup
    conn.free_gc(gc_white)?;
    conn.free_gc(gc_black)?;
    conn.free_pixmap(pixmap)?;

    Ok(())
}

fn get_cursor_position(conn: &RustConnection, root: u32) -> Result<(i16, i16)> {
    let reply = conn
        .query_pointer(root)?
        .reply()
        .context("query_pointer failed")?;
    Ok((reply.root_x, reply.root_y))
}

fn parse_hex_color(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16).unwrap_or(0x000000)
}

fn build_keymap(conn: &RustConnection) -> Result<std::collections::HashMap<u8, u32>> {
    let setup = conn.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;
    let count = max.saturating_sub(min).saturating_add(1);

    let reply = conn
        .get_keyboard_mapping(min, count)?
        .reply()
        .context("get_keyboard_mapping failed")?;

    if reply.keysyms_per_keycode == 0 {
        return Ok(std::collections::HashMap::new());
    }

    let per_kc = reply.keysyms_per_keycode as usize;
    let mut map = std::collections::HashMap::new();

    for (offset, keysyms) in reply.keysyms.chunks(per_kc).enumerate() {
        let keycode = min.saturating_add(offset as u8);
        if let Some(&ks) = keysyms.first()
            && ks != 0
        {
            map.insert(keycode, ks);
        }
    }

    Ok(map)
}
