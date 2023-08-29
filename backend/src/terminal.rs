use axum::extract::ws::{Message, WebSocket};
use bytes::BytesMut;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use wspty::PtyMaster;

/// # Window size of a terminal
///
/// The client sends this to request a PTY resize. This way the frontend xtermjs
/// instance fits to the helix instance.
#[derive(Deserialize, Debug)]
pub struct WindowSize {
    pub cols: u16,
    pub rows: u16,
}

/// # Send byte stream of PTY to client
///
/// Receives byte stream of PTY and redirects it to the client.
pub async fn write_to_websocket(
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
pub async fn handle_pty_incoming(
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
pub async fn handle_websocket_incoming(
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
