use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{debug, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::rust_connection::RustConnection;

use crate::config::{AlwaysOnTopConfig, parse_hex_color};

/// Commands sent from the main module to the border thread.
pub enum BorderCmd {
    Add(u32),
    Remove(u32),
    Shutdown,
}

/// The 4 border window IDs surrounding a target window.
struct BorderWindows {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
    /// Last known geometry of the target window.
    last_geom: (i32, i32, u32, u32),
}

impl BorderWindows {
    fn all(&self) -> [u32; 4] {
        [self.top, self.bottom, self.left, self.right]
    }
}

struct BorderManager {
    conn: RustConnection,
    root: u32,
    screen_depth: u8,
    borders: HashMap<u32, BorderWindows>,
    color_pixel: u32,
    opacity_value: u32,
    thickness: u32,
}

impl BorderManager {
    fn new(config: &AlwaysOnTopConfig) -> Result<Self> {
        let (conn, screen_num) =
            RustConnection::connect(None).context("border: failed to connect to X11")?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let screen_depth = screen.root_depth;

        let (r, g, b) = parse_hex_color(&config.border_color).unwrap_or((0, 120, 212));
        let color_pixel = (r as u32) << 16 | (g as u32) << 8 | b as u32;

        // _NET_WM_WINDOW_OPACITY is a 32-bit cardinal in range 0..u32::MAX
        let opacity_value = (config.border_opacity.clamp(0.0, 1.0) * u32::MAX as f32) as u32;

        Ok(Self {
            conn,
            root,
            screen_depth,
            borders: HashMap::new(),
            color_pixel,
            opacity_value,
            thickness: config.border_thickness.max(1),
        })
    }

    fn add(&mut self, target: u32) -> Result<()> {
        if self.borders.contains_key(&target) {
            return Ok(());
        }

        let geom = self.get_geometry(target)?;
        let wins = self.create_border_windows(target, geom)?;
        self.borders.insert(target, wins);
        Ok(())
    }

    fn remove(&mut self, target: u32) -> Result<()> {
        if let Some(bw) = self.borders.remove(&target) {
            for wid in bw.all() {
                let _ = self.conn.destroy_window(wid);
            }
            self.conn.flush().context("flush after destroy")?;
        }
        Ok(())
    }

    fn remove_all(&mut self) {
        let targets: Vec<u32> = self.borders.keys().copied().collect();
        for t in targets {
            let _ = self.remove(t);
        }
    }

    fn create_border_windows(
        &self,
        _target: u32,
        (x, y, w, h): (i32, i32, u32, u32),
    ) -> Result<BorderWindows> {
        let t = self.thickness;

        // Compute positions for 4 border strips
        let positions: [(i16, i16, u16, u16); 4] = [
            // top: spans full width including corners
            (
                (x - t as i32) as i16,
                (y - t as i32) as i16,
                (w + 2 * t) as u16,
                t as u16,
            ),
            // bottom
            (
                (x - t as i32) as i16,
                (y + h as i32) as i16,
                (w + 2 * t) as u16,
                t as u16,
            ),
            // left
            ((x - t as i32) as i16, y as i16, t as u16, h as u16),
            // right
            ((x + w as i32) as i16, y as i16, t as u16, h as u16),
        ];

        let mut ids = [0u32; 4];
        for (i, &(px, py, pw, ph)) in positions.iter().enumerate() {
            let wid = self.conn.generate_id().context("generate X11 id")?;

            let values = xproto::CreateWindowAux::new()
                .background_pixel(self.color_pixel)
                .override_redirect(1)
                .event_mask(xproto::EventMask::EXPOSURE);

            self.conn
                .create_window(
                    self.screen_depth,
                    wid,
                    self.root,
                    px,
                    py,
                    pw.max(1),
                    ph.max(1),
                    0,
                    xproto::WindowClass::INPUT_OUTPUT,
                    0,
                    &values,
                )
                .context("create border window")?;

            // Raise to top of stacking order
            self.raise_window(wid)?;

            // Set opacity
            self.set_opacity(wid)?;

            // Map the window
            self.conn.map_window(wid).context("map border window")?;

            ids[i] = wid;
        }

        self.conn.flush().context("flush after create borders")?;

        Ok(BorderWindows {
            top: ids[0],
            bottom: ids[1],
            left: ids[2],
            right: ids[3],
            last_geom: (x, y, w, h),
        })
    }

    /// Raise an override-redirect window to the top of the stacking order.
    /// Uses configure_window(stack_mode=ABOVE) which works for OR windows,
    /// unlike _NET_WM_STATE_ABOVE client messages that the WM ignores.
    fn raise_window(&self, wid: u32) -> Result<()> {
        let values = xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE);
        self.conn
            .configure_window(wid, &values)
            .context("raise border window")?;
        Ok(())
    }

    fn set_opacity(&self, wid: u32) -> Result<()> {
        let opacity_atom = self.intern_atom(b"_NET_WM_WINDOW_OPACITY")?;

        self.conn
            .change_property(
                xproto::PropMode::REPLACE,
                wid,
                opacity_atom,
                xproto::AtomEnum::CARDINAL,
                32,
                1,
                &self.opacity_value.to_ne_bytes(),
            )
            .context("set opacity")?;

        Ok(())
    }

    fn get_geometry(&self, window: u32) -> Result<(i32, i32, u32, u32)> {
        let geom = self
            .conn
            .get_geometry(window)
            .context("get_geometry failed")?
            .reply()
            .context("get_geometry reply failed")?;

        let coords = self
            .conn
            .translate_coordinates(window, self.root, 0, 0)
            .context("translate_coordinates failed")?
            .reply()
            .context("translate_coordinates reply failed")?;

        Ok((
            coords.dst_x as i32,
            coords.dst_y as i32,
            geom.width as u32,
            geom.height as u32,
        ))
    }

    fn update_positions(&mut self) {
        let targets: Vec<u32> = self.borders.keys().copied().collect();
        let mut to_remove = Vec::new();

        for target in targets {
            // Check if target window still exists
            let geom = match self.get_geometry(target) {
                Ok(g) => g,
                Err(_) => {
                    debug!("target window {target} gone, removing border");
                    to_remove.push(target);
                    continue;
                }
            };

            let bw = match self.borders.get_mut(&target) {
                Some(bw) => bw,
                None => continue,
            };

            let geom_changed = bw.last_geom != geom;

            let (x, y, w, h) = geom;
            let t = self.thickness;

            if geom_changed {
                let positions: [(i32, i32, u32, u32); 4] = [
                    (x - t as i32, y - t as i32, w + 2 * t, t),
                    (x - t as i32, y + h as i32, w + 2 * t, t),
                    (x - t as i32, y, t, h),
                    (x + w as i32, y, t, h),
                ];

                for (wid, &(px, py, pw, ph)) in bw.all().iter().zip(positions.iter()) {
                    let values = xproto::ConfigureWindowAux::new()
                        .x(px)
                        .y(py)
                        .width(pw.max(1))
                        .height(ph.max(1))
                        .stack_mode(xproto::StackMode::ABOVE);

                    let _ = self.conn.configure_window(*wid, &values);
                }

                bw.last_geom = geom;
            } else {
                // Even if geometry hasn't changed, re-raise borders so they
                // stay on top after focus/stacking changes.
                for wid in bw.all() {
                    let values =
                        xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE);
                    let _ = self.conn.configure_window(wid, &values);
                }
            }
        }

        for t in to_remove {
            let _ = self.remove(t);
        }

        let _ = self.conn.flush();
    }

    fn intern_atom(&self, name: &[u8]) -> Result<u32> {
        let reply = self
            .conn
            .intern_atom(false, name)
            .context("intern_atom request failed")?
            .reply()
            .context("intern_atom reply failed")?;
        Ok(reply.atom)
    }
}

/// Spawn the border management thread. Returns a sender to communicate with it.
pub fn spawn_border_thread(config: &AlwaysOnTopConfig) -> Result<mpsc::Sender<BorderCmd>> {
    let (tx, rx) = mpsc::channel();

    let mut manager = BorderManager::new(config)?;

    std::thread::Builder::new()
        .name("aot-borders".into())
        .spawn(move || {
            loop {
                // Process all pending commands (non-blocking)
                loop {
                    match rx.try_recv() {
                        Ok(BorderCmd::Add(w)) => {
                            if let Err(e) = manager.add(w) {
                                warn!("border add failed for {w}: {e}");
                            }
                        }
                        Ok(BorderCmd::Remove(w)) => {
                            if let Err(e) = manager.remove(w) {
                                warn!("border remove failed for {w}: {e}");
                            }
                        }
                        Ok(BorderCmd::Shutdown) => {
                            manager.remove_all();
                            return;
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            manager.remove_all();
                            return;
                        }
                    }
                }

                manager.update_positions();
                std::thread::sleep(Duration::from_millis(100));
            }
        })
        .context("failed to spawn border thread")?;

    Ok(tx)
}
