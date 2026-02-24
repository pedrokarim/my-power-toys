use anyhow::{Context, Result};
use tracing::info;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::rust_connection::RustConnection;

/// Get the currently focused X11 window ID.
pub fn get_focused_window() -> Result<u32> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let focus = conn
        .get_input_focus()
        .context("get_input_focus request failed")?
        .reply()
        .context("get_input_focus reply failed")?;

    let window = focus.focus;
    if window == root || window == 0 {
        anyhow::bail!("no focused window found");
    }
    Ok(window)
}

/// Get the root window (screen) dimensions.
/// For MVP this returns the full virtual desktop size (single-monitor assumption).
pub fn get_screen_geometry() -> Result<(u32, u32)> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    Ok((
        screen.width_in_pixels as u32,
        screen.height_in_pixels as u32,
    ))
}

/// Move and resize a window to the given absolute pixel coordinates.
///
/// Uses `_NET_MOVERESIZE_WINDOW` EWMH client message which is better than
/// raw `configure_window` because the window manager handles frame offsets.
pub fn move_resize_window(window: u32, x: i32, y: i32, w: u32, h: u32) -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // First remove maximized state, otherwise the WM ignores resize requests
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

    info!("Moved window {window} to ({x}, {y}) {w}x{h}");
    Ok(())
}

/// Remove _NET_WM_STATE_MAXIMIZED_HORZ and _VERT so the window can be freely resized.
fn unmaximize_window(conn: &RustConnection, root: u32, window: u32) -> Result<()> {
    let net_wm_state = intern_atom(conn, b"_NET_WM_STATE")?;
    let max_h = intern_atom(conn, b"_NET_WM_STATE_MAXIMIZED_HORZ")?;
    let max_v = intern_atom(conn, b"_NET_WM_STATE_MAXIMIZED_VERT")?;

    // Action 0 = remove
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

fn intern_atom(conn: &RustConnection, name: &[u8]) -> Result<u32> {
    let reply = conn
        .intern_atom(false, name)
        .context("intern_atom request failed")?
        .reply()
        .context("intern_atom reply failed")?;
    Ok(reply.atom)
}
