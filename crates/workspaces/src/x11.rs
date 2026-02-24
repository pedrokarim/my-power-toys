use anyhow::{Context, Result};
use tracing::debug;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, Atom, ConnectionExt};
use x11rb::rust_connection::RustConnection;

/// Information about a captured window.
#[derive(Debug, Clone)]
pub struct CapturedWindow {
    pub window_id: u32,
    pub title: String,
    pub wm_class: String,
    pub pid: Option<u32>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// List all managed (normal) application windows on the desktop.
pub fn list_windows() -> Result<Vec<CapturedWindow>> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Get the client list from the WM (more reliable than query_tree)
    let client_list = get_client_list(&conn, root)?;

    let mut windows = Vec::new();
    for win in client_list {
        if !is_normal_window(&conn, win)? {
            continue;
        }

        let title = get_window_title(&conn, win).unwrap_or_default();
        let wm_class = match get_wm_class(&conn, win) {
            Ok(c) => c,
            Err(_) => continue, // skip windows without WM_CLASS
        };
        let pid = get_window_pid(&conn, win).ok().flatten();
        let (x, y, width, height) = get_window_geometry(&conn, root, win)?;

        debug!("Window {win}: {title:?} class={wm_class:?} pid={pid:?} {width}x{height}+{x}+{y}");

        windows.push(CapturedWindow {
            window_id: win,
            title,
            wm_class,
            pid,
            x,
            y,
            width,
            height,
        });
    }

    Ok(windows)
}

/// Move and resize a window to the given absolute pixel coordinates.
pub fn move_resize_window(window: u32, x: i32, y: i32, w: u32, h: u32) -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Remove maximized state first
    unmaximize_window(&conn, root, window)?;

    let net_moveresize = intern_atom(&conn, b"_NET_MOVERESIZE_WINDOW")?;

    // Flags: gravity=0 (NorthWest), source=2 (pager), x/y/w/h all present (bits 8-11)
    let flags: u32 = (0xF << 8) | (2 << 12);

    let event = xproto::ClientMessageEvent::new(
        32,
        window,
        net_moveresize,
        [flags, x as u32, y as u32, w, h],
    );

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_REDIRECT | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .context("failed to send _NET_MOVERESIZE_WINDOW")?;

    conn.flush().context("failed to flush X11 connection")?;
    Ok(())
}

/// Find windows matching a given WM_CLASS.
pub fn find_windows_by_class(wm_class: &str) -> Result<Vec<u32>> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let client_list = get_client_list(&conn, root)?;
    let mut matches = Vec::new();

    for win in client_list {
        if let Ok(cls) = get_wm_class(&conn, win)
            && cls.eq_ignore_ascii_case(wm_class)
        {
            matches.push(win);
        }
    }

    Ok(matches)
}

// ── Internal helpers ────────────────────────────────────────────────────

/// Get _NET_CLIENT_LIST from the root window.
fn get_client_list(conn: &RustConnection, root: u32) -> Result<Vec<u32>> {
    let atom = intern_atom(conn, b"_NET_CLIENT_LIST")?;

    let reply = conn
        .get_property(false, root, atom, xproto::AtomEnum::WINDOW, 0, 1024)
        .context("get _NET_CLIENT_LIST request failed")?
        .reply()
        .context("get _NET_CLIENT_LIST reply failed")?;

    if reply.format == 32 {
        Ok(reply
            .value32()
            .map(|iter| iter.collect())
            .unwrap_or_default())
    } else {
        Ok(Vec::new())
    }
}

/// Check if a window is a normal application window.
fn is_normal_window(conn: &RustConnection, window: u32) -> Result<bool> {
    let wm_type_atom = intern_atom(conn, b"_NET_WM_WINDOW_TYPE")?;
    let normal_atom = intern_atom(conn, b"_NET_WM_WINDOW_TYPE_NORMAL")?;

    let reply = conn
        .get_property(false, window, wm_type_atom, xproto::AtomEnum::ATOM, 0, 32)
        .context("get _NET_WM_WINDOW_TYPE failed")?
        .reply()
        .context("get _NET_WM_WINDOW_TYPE reply failed")?;

    if reply.format == 32
        && let Some(atoms) = reply.value32()
    {
        let atoms: Vec<u32> = atoms.collect();
        return Ok(atoms.contains(&normal_atom));
    }

    // If no type is set, assume normal (per EWMH spec)
    Ok(true)
}

/// Get _NET_WM_NAME (UTF-8) or fallback to WM_NAME.
fn get_window_title(conn: &RustConnection, window: u32) -> Result<String> {
    // Try _NET_WM_NAME first (UTF-8)
    let net_wm_name = intern_atom(conn, b"_NET_WM_NAME")?;
    let utf8_string = intern_atom(conn, b"UTF8_STRING")?;

    let reply = conn
        .get_property(false, window, net_wm_name, utf8_string, 0, 256)
        .context("get _NET_WM_NAME failed")?
        .reply()
        .context("get _NET_WM_NAME reply failed")?;

    if reply.value_len > 0 {
        return Ok(String::from_utf8_lossy(&reply.value).to_string());
    }

    // Fallback to WM_NAME
    let reply = conn
        .get_property(
            false,
            window,
            xproto::AtomEnum::WM_NAME,
            xproto::AtomEnum::STRING,
            0,
            256,
        )
        .context("get WM_NAME failed")?
        .reply()
        .context("get WM_NAME reply failed")?;

    Ok(String::from_utf8_lossy(&reply.value).to_string())
}

/// Get WM_CLASS instance name (first part of WM_CLASS property).
fn get_wm_class(conn: &RustConnection, window: u32) -> Result<String> {
    let reply = conn
        .get_property(
            false,
            window,
            xproto::AtomEnum::WM_CLASS,
            xproto::AtomEnum::STRING,
            0,
            256,
        )
        .context("get WM_CLASS failed")?
        .reply()
        .context("get WM_CLASS reply failed")?;

    if reply.value.is_empty() {
        anyhow::bail!("empty WM_CLASS");
    }

    // WM_CLASS is two null-terminated strings: instance\0class\0
    // We use the instance name (first part) for matching
    let value = String::from_utf8_lossy(&reply.value);
    let instance = value.split('\0').next().unwrap_or("");

    if instance.is_empty() {
        anyhow::bail!("empty WM_CLASS instance");
    }

    Ok(instance.to_string())
}

/// Get _NET_WM_PID for a window.
fn get_window_pid(conn: &RustConnection, window: u32) -> Result<Option<u32>> {
    let atom = intern_atom(conn, b"_NET_WM_PID")?;

    let reply = conn
        .get_property(false, window, atom, xproto::AtomEnum::CARDINAL, 0, 1)
        .context("get _NET_WM_PID failed")?
        .reply()
        .context("get _NET_WM_PID reply failed")?;

    if reply.format == 32
        && let Some(mut values) = reply.value32()
    {
        return Ok(values.next());
    }

    Ok(None)
}

/// Get window geometry in absolute screen coordinates.
fn get_window_geometry(
    conn: &RustConnection,
    root: u32,
    window: u32,
) -> Result<(i32, i32, u32, u32)> {
    // Get size from get_geometry
    let geom = conn
        .get_geometry(window)
        .context("get_geometry failed")?
        .reply()
        .context("get_geometry reply failed")?;

    // Get absolute position via translate_coordinates
    let coords = conn
        .translate_coordinates(window, root, 0, 0)
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

/// Remove maximized state so the window can be freely resized.
fn unmaximize_window(conn: &RustConnection, root: u32, window: u32) -> Result<()> {
    let net_wm_state = intern_atom(conn, b"_NET_WM_STATE")?;
    let max_h = intern_atom(conn, b"_NET_WM_STATE_MAXIMIZED_HORZ")?;
    let max_v = intern_atom(conn, b"_NET_WM_STATE_MAXIMIZED_VERT")?;

    let event = xproto::ClientMessageEvent::new(32, window, net_wm_state, [0, max_h, max_v, 1, 0]);

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_REDIRECT | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .context("failed to send unmaximize")?;

    conn.flush()?;
    Ok(())
}

fn intern_atom(conn: &RustConnection, name: &[u8]) -> Result<Atom> {
    let reply = conn
        .intern_atom(false, name)
        .context("intern_atom request failed")?
        .reply()
        .context("intern_atom reply failed")?;
    Ok(reply.atom)
}
