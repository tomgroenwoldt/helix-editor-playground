use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use bytes::BytesMut;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use rnglib::RNG;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tracing::info;
use wspty::{PtyCommand, PtyMaster};

use crate::error::AppError;

/// # Window size of a terminal
///
/// The client sends this to request a PTY resize. This way the frontend xtermjs
/// instance fits to the helix instance.
#[derive(Deserialize, Debug)]
pub struct WindowSize {
    pub cols: u16,
    pub rows: u16,
}

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
    let mut cmd = Command::new("podman");
    cmd.arg("run")
        .arg("--rm")
        .arg("--env")
        .arg("HELIX=release")
        .arg("--env")
        .arg("ARGS=--version")
        .arg("helix-container:latest");
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
    let mut cmd = Command::new("podman");
    cmd.arg("run")
        .arg("--rm")
        .arg("--env")
        .arg("HELIX=master")
        .arg("--env")
        .arg("ARGS=--version")
        .arg("helix-container:latest");
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

/// # Upgrade connection to a websocket serving helix
///
/// The version of helix depends on the passed in path variable.
pub async fn editor(ws: WebSocketUpgrade, Path(version): Path<String>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, version))
}

/// # Serve helix via websocket
///
/// Runs helix --tutor inside a container and serves the stdout of the PTY via
/// websocket to the client. Receives byte stream of xtermjs terminal and pipes
/// it into the PTY.
pub async fn handle_ws(ws: WebSocket, version: String) {
    let (ws_outgoing, ws_incoming) = ws.split();
    let (sender, receiver) = unbounded_channel();
    let ws_sender = sender.clone();

    // Create a random container name
    let rng =
        RNG::try_from(&rnglib::Language::Roman).expect("Error creating the random name generator.");
    let container_name = format!("{}-{}", rng.generate_name(), rng.generate_name());

    if !(version == "release" || version == "master") {
        panic!("Unsupported version for helix");
    }

    // Setup a PTY running a containerized helix
    info!(
        "Starting container helix-{} in {} mode",
        &container_name, version
    );
    let mut cmd = Command::new("podman");
    cmd.arg("run")
        .arg("--name")
        .arg(format!("helix-{}", &container_name))
        .arg("--network")
        .arg("none")
        .arg("-it")
        .arg("--env")
        .arg(format!("HELIX={version}"))
        .arg("--env")
        .arg("ARGS=--tutor")
        .arg("helix-container:latest");

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

    // Remove container by force after the connection has broken down
    info!("Forcefully removing container helix-{}", &container_name);
    Command::new("podman")
        .arg("rm")
        .arg("-f")
        .arg(format!("helix-{}", &container_name))
        .output()
        .await
        .expect("Unable to kill helix container.");
}

/// # Send byte stream of PTY to client
///
/// Receives byte stream of PTY and redirects it to the client.
async fn write_to_websocket(
    mut outgoing: SplitSink<WebSocket, Message>,
    mut receiver: UnboundedReceiver<Message>,
) -> Result<(), anyhow::Error> {
    while let Some(msg) = receiver.recv().await {
        outgoing.send(msg).await?;
    }
    Ok(())
}

/// # Read stdout of PTY and send it to client
///
/// Constantly reads the stdout of the running helix instance and redirects it
/// to websocket sending channel.
async fn handle_pty_incoming(
    mut pty_shell_reader: PtyMaster,
    websocket_sender: UnboundedSender<Message>,
) -> Result<(), anyhow::Error> {
    let mut buffer = BytesMut::with_capacity(4096);
    buffer.resize(4096, 0u8);
    loop {
        buffer[0] = 0u8;
        let mut tail = &mut buffer[1..];
        let n = pty_shell_reader.read_buf(&mut tail).await?;
        if n == 0 {
            break;
        }
        websocket_sender.send(Message::Binary(buffer[..n + 1].to_vec()))?;
    }
    Ok(())
}

/// # Forward client messages to PTY
///
/// Sends client messages to running helix instance. Additionally handles ping
/// message.
async fn handle_websocket_incoming(
    mut incoming: SplitStream<WebSocket>,
    mut pty_shell_writer: PtyMaster,
    websocket_sender: UnboundedSender<Message>,
    stop_sender: UnboundedSender<()>,
) -> Result<(), anyhow::Error> {
    while let Some(Ok(msg)) = incoming.next().await {
        if let Message::Binary(msg) = msg {
            let data = msg;
            match data[0] {
                // Byte stream
                0 => {
                    if data.len().gt(&0) {
                        pty_shell_writer.write_all(&data[1..]).await?;
                    }
                }
                // Resize
                1 => {
                    let resize_msg: WindowSize = serde_json::from_slice(&data[1..])?;
                    pty_shell_writer.resize(resize_msg.cols, resize_msg.rows)?;
                }
                _ => (),
            }
        } else if let Message::Ping(msg) = msg {
            let data = msg;
            websocket_sender.send(Message::Pong(data))?
        }
    }
    stop_sender.send(())?;
    Ok(())
}
