use std::path::PathBuf;

use mpt_common::ipc;

use crate::message::DaemonStateResult;

/// Check daemon status and fetch module list via D-Bus.
pub async fn poll_daemon() -> DaemonStateResult {
    let conn = match zbus::Connection::session().await {
        Ok(c) => c,
        Err(_) => {
            return DaemonStateResult {
                connected: false,
                modules: vec![],
            };
        }
    };

    if conn
        .call_method(
            Some(ipc::BUS_NAME),
            ipc::OBJECT_PATH,
            Some(ipc::BUS_NAME),
            "Ping",
            &(),
        )
        .await
        .is_err()
    {
        return DaemonStateResult {
            connected: false,
            modules: vec![],
        };
    }

    let modules = match conn
        .call_method(
            Some(ipc::BUS_NAME),
            ipc::OBJECT_PATH,
            Some(ipc::BUS_NAME),
            "ListModules",
            &(),
        )
        .await
    {
        Ok(msg) => msg
            .body()
            .deserialize::<Vec<(String, String, bool)>>()
            .unwrap_or_default(),
        Err(_) => vec![],
    };

    DaemonStateResult {
        connected: true,
        modules,
    }
}

pub async fn daemon_toggle_module(id: String, enable: bool) -> (String, bool, String) {
    let method = if enable { "StartModule" } else { "StopModule" };
    let result = async {
        let conn = zbus::Connection::session().await?;
        let msg = conn
            .call_method(
                Some(ipc::BUS_NAME),
                ipc::OBJECT_PATH,
                Some(ipc::BUS_NAME),
                method,
                &id.as_str(),
            )
            .await?;
        let r: String = msg.body().deserialize()?;
        Ok::<String, zbus::Error>(r)
    }
    .await;

    let status = match result {
        Ok(s) => s,
        Err(e) => format!("error: {e}"),
    };
    (id, enable, status)
}

pub async fn pick_image_file() -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter("Images", &["jpg", "jpeg", "png", "bmp", "gif", "webp"])
        .set_title("Choose background image")
        .pick_file()
        .await?;
    Some(handle.path().to_path_buf())
}
