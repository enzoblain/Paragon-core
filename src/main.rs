use core::adapters::channel_adapter::ChannelAdapter;
use core::application::candle::consume::consume_candles;
use core::application::candle::publish::publish_candles;
use core::domain::{entities::candle::Candle, ports::{DataReceiver, DataSender}};

use tokio_scoped::scope;

#[tokio::main]
async fn main() {
    let adapter = ChannelAdapter::new(1); // Because we only have EUR/USD

    let candle_sender: &dyn DataSender<Candle> = &adapter;
    let candle_receiver: &dyn DataReceiver<Candle> = &adapter;

	scope(|s| {
		s.spawn(async move {
			publish_candles(candle_sender).await;
		});

		s.spawn(async move {
			consume_candles(candle_receiver).await;
		});
	});
}