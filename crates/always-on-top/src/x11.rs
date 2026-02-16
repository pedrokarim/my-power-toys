use anyhow::{Context, Result};
use tracing::info;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use x11rb::rust_connection::RustConnection;

/// Toggle _NET_WM_STATE_ABOVE on the currently focused X11 window.
pub fn toggle_always_on_top() -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None).context("failed to connect to X11")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Get the currently focused window
    let focus = conn
        .get_input_focus()
        .context("get_input_focus request failed")?
        .reply()
        .context("get_input_focus reply failed")?;

    let window = focus.focus;
    if window == root || window == 0 {
        anyhow::bail!("no focused window found");
    }

    // Intern the atoms we need
    let net_wm_state = intern_atom(&conn, b"_NET_WM_STATE")?;
    let net_wm_state_above = intern_atom(&conn, b"_NET_WM_STATE_ABOVE")?;

    // _NET_WM_STATE toggle action = 2
    const NET_WM_STATE_TOGGLE: u32 = 2;

    // Send a client message to the root window to toggle the state
    let event = xproto::ClientMessageEvent::new(
        32,
        window,
        net_wm_state,
        [NET_WM_STATE_TOGGLE, net_wm_state_above, 0, 1, 0],
    );

    conn.send_event(
        false,
        root,
        xproto::EventMask::SUBSTRUCTURE_REDIRECT | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .context("failed to send client message")?;

    conn.flush().context("failed to flush X11 connection")?;

    info!("Toggled always-on-top for window {window}");
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
