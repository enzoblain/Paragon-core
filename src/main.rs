use core::adapters::channel_adapter::ChannelAdapter;
use core::adapters::rest_data_inserter::RestDataInserter;
use core::adapters::websocket_data_sender::WebsocketDataSender;
use core::application::candle::consume::consume_candles;
use core::application::candle::publish::publish_candles;
use core::application::context::AppContext;
use core::application::data::consume::{insert_data, send_data};
use core::domain::entities::candle::Candle;
use core::domain::entities::data::Data;
use core::domain::ports::{DataReceiver, DataSender};

use std::sync::Arc;
use tokio_scoped::scope;

#[tokio::main]
async fn main() {
    let candle_adapter = ChannelAdapter::new(1); // Because we only have EUR/USD
    let candle_sender: &dyn DataSender<Candle> = &candle_adapter;
    let candle_receiver: &dyn DataReceiver<Candle> = &candle_adapter;

    let websocket_adapter = Arc::new(ChannelAdapter::new(16));
    let websocket_receiver: &dyn DataReceiver<Arc<Data>> = &websocket_adapter;
    let data_sender = WebsocketDataSender::new("ws://localhost:8080/ws".into());

    let rest_adapter = Arc::new(ChannelAdapter::new(16));
    let rest_receiver: &dyn DataReceiver<Arc<Data>> = &rest_adapter;
    let data_inserter = RestDataInserter::new("http://localhost:4000/graphql".into());
    let ctx = AppContext::new(rest_adapter.clone(), websocket_adapter.clone());

    scope(|s| {
        s.spawn(async move {
            publish_candles(candle_sender).await;
        });

        s.spawn(async move {
            consume_candles(&ctx, candle_receiver).await;
        });

        s.spawn(async move {
            send_data(websocket_receiver, data_sender).await;
        });

        s.spawn(async move {
            insert_data(rest_receiver, data_inserter).await;
        });
    });
}
