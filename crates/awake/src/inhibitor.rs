use anyhow::{Context, Result};
use tracing::info;
use zbus::blocking::Connection;

/// Holds D-Bus screensaver/suspend inhibit cookies.
/// Dropping or calling `release()` un-inhibits.
pub struct ScreenSaverInhibitor {
    conn: Connection,
    screensaver_cookie: Option<u32>,
    login_fd: Option<zbus::zvariant::OwnedFd>,
}

impl ScreenSaverInhibitor {
    /// Inhibit screensaver and/or suspend via D-Bus.
    ///
    /// Uses two interfaces:
    /// - `org.freedesktop.ScreenSaver.Inhibit` — prevents screen blanking (only when `keep_screen_on`)
    /// - `org.freedesktop.login1.Manager.Inhibit` — prevents system suspend
    pub fn inhibit(keep_screen_on: bool) -> Result<Self> {
        let conn =
            Connection::session().context("failed to connect to session D-Bus for screensaver")?;

        // 1) Inhibit screensaver (only if keep_screen_on is requested)
        let screensaver_cookie = if keep_screen_on {
            inhibit_screensaver(&conn).ok()
        } else {
            None
        };

        // 2) Inhibit suspend via logind (system bus)
        let login_fd = inhibit_suspend().ok();

        if screensaver_cookie.is_none() && login_fd.is_none() {
            anyhow::bail!("failed to inhibit both screensaver and suspend");
        }

        Ok(Self {
            conn,
            screensaver_cookie,
            login_fd,
        })
    }

    pub fn release(self) -> Result<()> {
        // Release screensaver inhibitor
        if let Some(cookie) = self.screensaver_cookie {
            if let Err(e) = uninhibit_screensaver(&self.conn, cookie) {
                tracing::warn!("Failed to release screensaver inhibitor: {e}");
            } else {
                info!("Released screensaver inhibitor");
            }
        }

        // Suspend inhibitor is released by dropping the fd
        if self.login_fd.is_some() {
            info!("Released suspend inhibitor (fd dropped)");
        }

        Ok(())
    }
}

/// Call org.freedesktop.ScreenSaver.Inhibit(app_name, reason) -> cookie
fn inhibit_screensaver(conn: &Connection) -> Result<u32> {
    let reply = conn.call_method(
        Some("org.freedesktop.ScreenSaver"),
        "/org/freedesktop/ScreenSaver",
        Some("org.freedesktop.ScreenSaver"),
        "Inhibit",
        &("MyPowerToys", "User requested Awake mode"),
    )?;

    let cookie: u32 = reply.body().deserialize()?;
    info!("Screensaver inhibited (cookie={cookie})");
    Ok(cookie)
}

/// Call org.freedesktop.ScreenSaver.UnInhibit(cookie)
fn uninhibit_screensaver(conn: &Connection, cookie: u32) -> Result<()> {
    conn.call_method(
        Some("org.freedesktop.ScreenSaver"),
        "/org/freedesktop/ScreenSaver",
        Some("org.freedesktop.ScreenSaver"),
        "UnInhibit",
        &(cookie,),
    )?;
    Ok(())
}

/// Inhibit suspend via systemd-logind.
/// Returns an fd that must be kept open to maintain the inhibition.
fn inhibit_suspend() -> Result<zbus::zvariant::OwnedFd> {
    let conn = Connection::system().context("failed to connect to system D-Bus for logind")?;

    let reply = conn.call_method(
        Some("org.freedesktop.login1"),
        "/org/freedesktop/login1",
        Some("org.freedesktop.login1.Manager"),
        "Inhibit",
        &(
            "idle:sleep",                // what
            "MyPowerToys",               // who
            "User requested Awake mode", // why
            "block",                     // mode
        ),
    )?;

    let fd: zbus::zvariant::OwnedFd = reply.body().deserialize()?;
    info!("Suspend inhibited (logind fd acquired)");
    Ok(fd)
}
