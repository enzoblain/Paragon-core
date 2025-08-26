use crate::domain::{entities::Candle, ports::DataReceiver};

pub async fn consume_candles<S: DataReceiver<Candle> + ?Sized>(
    receiver: &S,
) {
    while let Some(candle) = receiver.receive_data().await {
        println!("{:?}", candle);
    }
}