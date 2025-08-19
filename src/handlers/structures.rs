use crate::{
    connections::{
        database::add_two_d_structure,
        websocket::send_data
    },
    Candle,
    OneDStructures, 
    Timerange,
    TwoDStructures
};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde_json::{Map, to_value, Value};
use std::sync::Arc;

pub static LAST_THREE_CANDLES: Lazy<Arc<DashMap<String, Vec<Arc<Candle>>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});

pub async fn send_two_d_structure(structure: &TwoDStructures) -> Result<(), String> {
    let mut data = Map::new();

    data.insert("type".to_string(), Value::String("Two dimension structure".to_string())); 
    data.insert("value".to_string(), to_value(structure).unwrap());

    let json_data = Value::Object(data).to_string();

    send_data(json_data).await?;

    Ok(())
}

pub async fn send_one_d_structure(structure: &OneDStructures) -> Result<(), String> {
    let mut data = Map::new();

    data.insert("type".to_string(), Value::String("One dimension structure".to_string())); 
    data.insert("value".to_string(), to_value(structure).unwrap());

    let json_data = Value::Object(data).to_string();

    send_data(json_data).await?;

    Ok(())
}

pub async fn processfairvaluegap(candle: Arc<Candle>, symbol: &'static str, timerange: &Timerange) -> Result<(), String> {
    let key = format!("{}-{}", symbol, timerange.label);

    let last_candles = LAST_THREE_CANDLES
        .get_mut(key.as_str());

    if let Some(mut last_candles) = last_candles {
        last_candles.push(candle.clone());

        // We only need 3 candles (if less, then we pass)
        if last_candles.len() > 4 {
            return Err("Too many candles in the list".to_string());
        } else if last_candles.len() == 4 {
            last_candles.remove(0);
        } else if last_candles.len() < 3 {
            return Ok(());
        }

        // Now we have to check if all the candles have the same direction
        // Because if they don't, we can't have a fair value gap
        let mut direction: Option<String> = None;
        for candle in last_candles.iter() {
            if let Some(ref direction) = direction {
                if *direction != candle.direction && candle.direction != "doji".to_string() {
                    return Ok(());
                }
            } else {
                // If not we initialize the direction
                // We ignore doji candles for the direction
                if *candle.direction != "doji".to_string() {
                    direction = Some(candle.direction.clone());
                }
            }
        }

        let mut high: Option<f64> = None;
        let mut low: Option<f64> = None;

        if let Some(direction) = direction.clone() {
            // If it's bullish, we have to find a hole between the first candle shadow and the third candle body
            if direction == "bullish" {
                if last_candles[0].high < last_candles[2].low {
                    high = Some(last_candles[2].low);
                    low = Some(last_candles[0].high);
                }
            // If it's bearish, we have to find a hole between the first candle body and the third candle shadow
            } else if direction == "bearish" {
                if last_candles[0].low > last_candles[2].high {
                    high = Some(last_candles[0].low);
                    low = Some(last_candles[2].high);
                }
            }
        }

        // If we have found a fair value gap, we create a TwoDStructures entity
        if let (Some(high), Some(low)) = (high, low) {
            let fair_value_gap = TwoDStructures {
                symbol: symbol.to_string(),
                structure: "Fair Value Gap".to_string(),
                timerange: timerange.label.to_string(),
                timestamp: candle.timestamp,
                high,
                low,
                direction: direction.unwrap_or("doji".to_string()), // But this should never happen
            };

            if let Err(e) = add_two_d_structure(&fair_value_gap) {
                eprintln!("Failed to add 2D structure to the database: {}", e);
            }

            send_two_d_structure(&fair_value_gap).await?;
        }
    } else {
        let mut new_candles = Vec::new();
        new_candles.push(candle);
        LAST_THREE_CANDLES.insert(key, new_candles);
    }

    Ok(())
}