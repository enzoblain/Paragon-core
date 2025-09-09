use crate::adapters::websocket_data_sender::WebsocketDataSender;
use crate::domain::entities::data::Data;
use crate::domain::ports::DataReceiver;

use futures_util::SinkExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn consume_data<S: DataReceiver<Data> + ?Sized>(
    receiver: &S,
    sender: WebsocketDataSender,
) {
    match connect_async(sender.req).await {
        Ok((mut ws_stream, _)) => {
            while let Some(data) = receiver.receive_data().await {
                ws_stream
                    .send(Message::Text(data.into_string().into()))
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Error sending data over WebSocket: {}", e);
                    });
            }
        }
        Err(e) => {
            eprintln!("WebSocket connection error: {}", e);
            while receiver.receive_data().await.is_some() {
                eprintln!("WebSocket not connected. data not sent");
            }
        }
    }
}
