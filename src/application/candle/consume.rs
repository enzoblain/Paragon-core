use crate::application::structures::fvg::process_fvg;
use crate::application::structures::session::process_session;
use crate::application::structures::trend::process_trend;
use crate::domain::entities::candle::{Candle, CANDLES};
use crate::domain::entities::timerange::{Timerange, TIMERANGES};
use crate::domain::ports::DataReceiver;

use tokio_scoped::scope;

pub async fn consume_candles<S: DataReceiver<Candle> + ?Sized>(receiver: &S) {
    while let Some(candle) = receiver.receive_data().await {
        scope(|s| {
            for timerange in TIMERANGES {
                s.spawn(async {
                    aggregate_candle(&candle, timerange).await;
                });
            }

            s.spawn(async {
                process_session(&candle).await;
            });
        });
    }
}

pub async fn aggregate_candle(candle: &Candle, timerange: &'static Timerange) {
    let key = (candle.symbol, timerange.label);

    if let Some(mut last_candle) = CANDLES.get_mut(&key) {
        // If the new candle's timestamp is within the current candle's time range
        if candle.timestamp >= last_candle.end_timestamp {
            // TODO: send the candle via websocket
            // TODO: add the candle to the db

            process_trend(&last_candle).await;
            process_fvg(&last_candle).await;

            *last_candle = Candle::new(
                candle.symbol,
                timerange,
                candle.timestamp,
                candle.open,
                candle.high,
                candle.low,
                candle.close,
                candle.volume,
            );
        } else {
            // If the new candle is in the same timerange
            last_candle.high = last_candle.high.max(candle.high);
            last_candle.low = last_candle.low.min(candle.low);
            last_candle.close = candle.close;
            last_candle.volume += candle.volume;
        }
    } else {
        // If there isn't candle stored in the actual timerange
        let new_candle = Candle::new(
            candle.symbol,
            timerange,
            candle.timestamp,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume,
        );

        CANDLES.insert(key, new_candle);
    }

    // TODO: send the candle via websocket
}
