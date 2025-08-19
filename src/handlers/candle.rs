use crate::{
    Candle,
    connections::{
        database::add_candle,
        websocket::send_data
    },
    handlers::{
        structures::processfairvaluegap,
        trends::process_trend
    },
    Timerange,
};

use chrono::{Utc, TimeZone};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde_json::{Map, to_value, Value};
use std::sync::Arc;

pub static CANDLES: Lazy<Arc<DashMap<String, Arc<Candle>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});

pub async fn aggregate_candle(candle: Arc<Candle>, symbol: &'static str, timerange: &Timerange) {
    let key = format!("{}-{}", symbol, timerange.label);

    let last_candle = CANDLES
        .get(key.as_str())
        .map(|c| Arc::clone(c.value()));

    let new_candle;

    // Check if it's the first candle for this timerange
    // If there is no last candle, we create a new one
    // If there is a last candle, we check if the new candle is in the same timerange 
    if let Some(last_candle) = last_candle {
        if last_candle.timestamp + chrono::Duration::milliseconds(timerange.duration_ms as i64) <= candle.timestamp {
            if let Err(e) = add_candle(&last_candle) {
                eprintln!("Failed to add candle to database: {}", e);
            }

            if let Err(e) = send_candle(&last_candle).await {
                eprintln!("Failed to send candle to websocket: {}", e);
            }

            if let Err(e) = processfairvaluegap(Arc::clone(&last_candle), symbol, timerange).await {
                eprintln!("Failed to process fair value gap: {}", e);
            }

            if let Err(e) = process_trend(Arc::clone(&last_candle), symbol, timerange.label).await {
                eprintln!("Failed to process trend: {}", e);
            }

            let mut modified_candle = (*candle).clone();
            modified_candle.timerange = timerange.label.to_string();

            // Adjust the open price to match the timerange,
            modified_candle.timestamp = Utc.timestamp_millis_opt((modified_candle.timestamp.timestamp_millis() / timerange.duration_ms as i64) * timerange.duration_ms as i64).single().expect("Failed to adjust timestamp");

            new_candle = Arc::new(modified_candle);
        } else {
            // If the new candle is in the same timerange
            // Take the last candle and update it with the new candle
            let mut modified_candle = (*last_candle).clone();

            // Update the candle with the new values
            modified_candle.high = modified_candle.high.max(candle.high);
            modified_candle.low = modified_candle.low.min(candle.low);
            modified_candle.close = candle.close;
            modified_candle.volume += candle.volume;

            new_candle = Arc::new(modified_candle);
        }
    } else {
        // Don't forget to change the timerange of the candle
        let mut candle = (*candle).clone();
        candle.timerange = timerange.label.to_string();

        // And adjust the timestamp to match the timerange
        // This is done by rounding the timestamp to the nearest timerange duration
        candle.timestamp = Utc.timestamp_millis_opt((candle.timestamp.timestamp_millis() / timerange.duration_ms as i64) * timerange.duration_ms as i64).single().expect("Failed to adjust timestamp");

        new_candle = Arc::new(candle);
    }

    if let Err(e) = send_candle(&new_candle).await {
        eprintln!("Failed to send candle to websocket: {}", e);
    }

    // Insert or update the candle in the DashMap
    CANDLES
        .entry(key)
        .and_modify(|c| *c = Arc::clone(&new_candle))
        .or_insert_with(|| Arc::clone(&new_candle));
}

pub async fn send_candle(candle: &Candle) -> Result<(), String> {
    let mut data = Map::new();

    // Structure the data
    data.insert("type".to_string(), Value::String("candle".to_string()));
    data.insert("value".to_string(), to_value(candle).unwrap());

    let json_data = Value::Object(data).to_string();

    send_data(json_data).await?;

    Ok(())
}