use crate::{
    connections::auth::get_jwt,
    utils::utils::generate_rand_base64
};
use common::{
    Config,
    utils::log::{
        LogFile,
        LogLevel
    }
};

use futures_util::SinkExt;
use http::{header::AUTHORIZATION, Request};
use std::sync::OnceLock;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message
};
use url::Url;

pub async fn websocket_connection(mut rx: Receiver<String>) -> Result<(), String> {
    let config = Config::global();
    let address = config.server.websocket.address.clone();
    let port = config.server.websocket.port;
    let full_address = format!("ws://{}:{}/ws", address, port);
    let url = Url::parse(full_address.as_str()).unwrap();

    let token = get_jwt("websocket".into()).map_err(|e| format!("Failed to get JWT: {}", e))?;

    let req = Request::builder()
        .uri(url.as_str())
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header("sec-websocket-key", generate_rand_base64())
        .header("host", format!("{}:{}", address, port))
        .header("connection", "Upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .body(())
        .map_err(|e| format!("Failed to build request: {}", e))?;

    match connect_async(req).await {
        Ok((mut ws_stream, _)) => {
            LogFile::add_log(LogLevel::Info, &format!("WebSocket connection established: {}", full_address)).ok();

            while let Some(data) = rx.recv().await {
                ws_stream.send(Message::Text(data))
                    .await
                    .map_err(|e| format!("Failed to send message: {}", e))?;
            }
        },
        Err(e) => {
            LogFile::add_log(LogLevel::Error, &format!("Failed to connect to WebSocket: {}", e)).ok();

            return Err(format!("Failed to connect: {}", e));
        }
    }
    
    Ok(())
}

pub static SENDER: OnceLock<Sender<String>> = OnceLock::new();

// This function creates a channel for sending messages
// Between the main application and the WebSocket connection
// It also initializes the sender
pub fn create_channel() -> Result<Receiver<String>, String> {
    let (tx, rx) = tokio::sync::mpsc::channel(256);

    SENDER.set(tx.clone()).map_err(|_| "Failed to set sender")?;

    Ok(rx)
}

pub async fn send_data(data: String) -> Result<(), String> {
    if let Some(sender) = SENDER.get() {
        sender.send(data).await.map_err(|e| format!("Failed to send data: {}", e))?;
        Ok(())
    } else {
        Err("Sender not initialized".into())
    }
}