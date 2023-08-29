use axum::{
    extract::{ws::WebSocket, Path, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::{process::Command, sync::mpsc::unbounded_channel};
use wspty::PtyCommand;

use crate::{
    error::AppError,
    terminal::{handle_pty_incoming, handle_websocket_incoming, write_to_websocket},
};

/// # Available versions of helix
///
/// Helix versions available in container image.
#[derive(Deserialize, Serialize)]
pub struct Versions {
    release: String,
    master: String,
}

/// # Fetch current helix versions
///
/// Runs "helix --version" inside the container and returns the output.
pub async fn get_versions() -> Result<Json<Versions>, AppError> {
    // Get release version
    let mut cmd = Command::new("helix");
    cmd.arg("--version");
    let output = cmd.output().await;
    let mut release = String::from("unknown");
    if let Ok(output) = output {
        release = String::from_utf8(output.stdout)?
            .split(' ')
            .last()
            .expect("Error parsing release version from podman output")
            .trim()
            .to_string();
    }

    // Get master version
    let mut cmd = Command::new("hx");
    cmd.arg("--version");
    let output = cmd.output().await;
    let mut master = String::from("unknown");
    if let Ok(output) = output {
        master = String::from_utf8(output.stdout)?
            .split(' ')
            .last()
            .expect("Error parsing master version from podman output")
            .replace(['(', ')'], "")
            .trim()
            .to_string();
    }

    Ok(Json(Versions { release, master }))
}

/// # Upgrade connection to a websocket serving helix tutor
pub async fn tutor(ws: WebSocketUpgrade, Path(version): Path<String>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, version))
}

/// # Serve helix via websocket
///
/// Runs helix --tutor inside a container and serves the stdout of the PTY via
/// websocket to the client. Receives byte stream of xtermjs terminal and pipes
/// it into the PTY.
pub async fn handle_ws(ws: WebSocket, version: String) {
    if !(version == "release" || version == "master") {}

    let helix = match version.as_str() {
        "release" => "helix",
        "master" => "hx",
        _ => panic!("Unsupported version for helix"),
    };

    let (ws_outgoing, ws_incoming) = ws.split();
    let (sender, receiver) = unbounded_channel();
    let ws_sender = sender.clone();

    let mut cmd = Command::new("bwrap");
    cmd.args([
        "--bind",
        "/usr/bin/sh",
        "/usr/bin/sh",
        "--bind",
        "/usr/bin/sleep",
        "/usr/bin/sleep",
        "--bind",
        &format!("/usr/bin/{}", helix),
        &format!("/usr/bin/{}", helix),
        "--ro-bind",
        "/lib/x86_64-linux-gnu/libgcc_s.so.1",
        "/lib/x86_64-linux-gnu/libgcc_s.so.1",
        "--ro-bind",
        "/lib/x86_64-linux-gnu/libm.so.6",
        "/lib/x86_64-linux-gnu/libm.so.6",
        "--ro-bind",
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib/x86_64-linux-gnu/libc.so.6",
        "--ro-bind",
        "/lib/x86_64-linux-gnu/libstdc++.so.6",
        "/lib/x86_64-linux-gnu/libstdc++.so.6",
        "--ro-bind",
        "/lib64/ld-linux-x86-64.so.2",
        "/lib64/ld-linux-x86-64.so.2",
        "--ro-bind",
        "/proc/self",
        "/proc/self",
        "--dir",
        "/tmp",
        "--tmpfs",
        "/home",
        "--tmpfs",
        "/tmp",
        "--die-with-parent",
        "--setenv",
        "HOME",
        "/home/user",
        "--setenv",
        "TERM",
        "xterm",
        "--unshare-user",
        "--uid",
        "1000",
        "--gid",
        "1000",
        "--bind",
        "/home/user/.config/helix",
        "/home/user/.config/helix",
        "--bind",
        "/home/user/playground",
        "/home/user/playground",
        "/usr/bin/sh",
        "-c",
        &format!(
            "sleep 0.5 && cd /home/user/playground && /usr/bin/{} --tutor",
            helix
        ),
    ]);

    let mut pty_cmd = PtyCommand::from(cmd);
    let (stop_sender, stop_receiver) = unbounded_channel();
    let pty_master = pty_cmd
        .run(stop_receiver)
        .await
        .expect("Error running helix in pty");
    let pty_shell_writer = pty_master.clone();
    let pty_shell_reader = pty_master.clone();

    tokio::select! {
        res = handle_websocket_incoming(ws_incoming, pty_shell_writer, sender, stop_sender) => res,
        res = handle_pty_incoming(pty_shell_reader, ws_sender) => res,
        res = write_to_websocket(ws_outgoing, receiver) => res,
    }
    .unwrap();
}
