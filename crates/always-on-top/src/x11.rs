use anyhow::{Context, Result};
use tracing::info;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::rust_connection::RustConnection;

/// Information about the currently focused X11 window.
pub struct FocusedWindow {
    pub id: u32,
    pub root: u32,
    pub wm_class: String,
}

/// Get the focused window with metadata needed for the toggle decision.
pub fn get_focused_window() -> Result<FocusedWindow> {
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

    let wm_class = get_wm_class(&conn, window).unwrap_or_default();

    Ok(FocusedWindow {
        id: window,
        root,
        wm_class,
    })
}

/// Explicitly set or remove _NET_WM_STATE_ABOVE on the given window.
/// Using explicit ADD/REMOVE instead of TOGGLE avoids desync between
/// our internal state tracking and the actual X11 window state.
pub fn set_always_on_top(window: u32, root: u32, enable: bool) -> Result<()> {
    let (conn, _) = RustConnection::connect(None).context("failed to connect to X11")?;

    let net_wm_state = intern_atom(&conn, b"_NET_WM_STATE")?;
    let net_wm_state_above = intern_atom(&conn, b"_NET_WM_STATE_ABOVE")?;

    const NET_WM_STATE_REMOVE: u32 = 0;
    const NET_WM_STATE_ADD: u32 = 1;

    let action = if enable {
        NET_WM_STATE_ADD
    } else {
        NET_WM_STATE_REMOVE
    };

    let event = xproto::ClientMessageEvent::new(
        32,
        window,
        net_wm_state,
        [action, net_wm_state_above, 0, 1, 0],
    );

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_REDIRECT | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .context("failed to send client message")?;

    conn.flush().context("failed to flush X11 connection")?;

    let verb = if enable { "Added" } else { "Removed" };
    info!("{verb} always-on-top for window {window}");
    Ok(())
}

/// Read WM_CLASS for a window (returns the instance name, first part).
pub fn get_wm_class(conn: &RustConnection, window: u32) -> Result<String> {
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
    let value = String::from_utf8_lossy(&reply.value);
    let instance = value.split('\0').next().unwrap_or("");

    if instance.is_empty() {
        anyhow::bail!("empty WM_CLASS instance");
    }

    Ok(instance.to_string())
}

fn intern_atom(conn: &RustConnection, name: &[u8]) -> Result<u32> {
    let reply = conn
        .intern_atom(false, name)
        .context("intern_atom request failed")?
        .reply()
        .context("intern_atom reply failed")?;
    Ok(reply.atom)
}
