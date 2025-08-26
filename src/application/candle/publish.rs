use crate::adapters::data_loader::{get_data, parse_candle};
use crate::domain::{entities::Candle, ports::DataSender};

use tokio::time::{sleep, Duration};

pub async fn publish_candles<S: DataSender<Candle> + ?Sized>(
    sender: &S,
) {
    let data = get_data().map_err(|e| e.to_string()).unwrap();

    for index in 0..data.height() {
        let row = data.get_row(index).map_err(|e| e.to_string()).unwrap();

        let candle = parse_candle(row).map_err(|e| e.to_string()).unwrap();

        if let Err(e) = sender.send_data(candle).await {
            eprintln!("Error sending data: {}", e);
        }

        sleep(Duration::from_secs(1)).await;
    }
}