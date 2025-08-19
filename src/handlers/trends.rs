// WARNING: This is the v1 of the trends recognition algorithm
// It may/has to changer and be reworked
// It may not be optimized and accurate

use crate::{
    connections::{
        database::{
            add_trend,
            add_one_d_structure,
            add_two_d_structure
        },
        websocket::send_data
    },
    handlers::structures::{
        send_one_d_structure,
        send_two_d_structure
    },
    Candle,
    OneDStructures,
    Subtrend, 
    Trend, 
    TwoDStructures
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde_json::{Map, to_value, Value};
use std::sync::Arc;

pub static QUEUE: Lazy<Arc<DashMap<String, Vec<Arc<Candle>>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});
pub static CANDLES: Lazy<Arc<DashMap<String, Arc<Candle>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});
pub static TRENDS: Lazy<Arc<DashMap<String, Arc<Trend>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});

pub static SUBTRENDS: Lazy<Arc<DashMap<String, Arc<Subtrend>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});

// This function sends a Trend entity to all connected clients via WebSocket
pub async fn send_trend(trend: &Trend) -> Result<(), String> {
    let mut data = Map::new();

    data.insert("type".to_string(), Value::String("Trend".to_string()));
    data.insert("value".to_string(), to_value(trend).unwrap());

    let json_data = Value::Object(data).to_string();

    send_data(json_data).await?;

    Ok(())
}

pub async fn process_trend(candle: Arc<Candle>, symbol: &'static str, timerange: &str) -> Result<(), String> {
    let key = format!("{}-{}", symbol, timerange);

    // Add the candle to the queue
    QUEUE.entry(key.clone()).or_default().push(candle.clone());

    let datetime = get_trends(candle).await;

    if let Some(datetime) = datetime? {
        process_queue(key.clone(), datetime)?;

        // To initialize and start the loop
        let mut processing_required = true;

        // Loop to process the queue until no more processing is required
        while processing_required {
            processing_required = false;

            // Collect the candles
            let candles: Vec<Arc<Candle>> = if let Some(queue) = QUEUE.get(&key) {
                queue.iter().cloned().collect()
            } else {
                Vec::new()
            };

            for candle in candles {
                // Process the candle to get the trends
                let datetime = get_trends(candle.clone()).await;

                // And check if we need to process the queue again
                if let Some(datetime) = datetime? {
                    processing_required = true;

                    process_queue(key.clone(), datetime)?;

                    break;
                }
            }
        }
    }

    Ok(())
}

// This function only keeps the candles that are after the given datetime
// Because the old candles are not relevant anymore
pub fn process_queue(key: String, datetime: DateTime<Utc>) -> Result<(), String> {
    match QUEUE.entry(key) {
        // If the entry exists, we retain only the candles that are after the datetime
        dashmap::mapref::entry::Entry::Occupied(mut entry) => {
            entry.get_mut().retain(|c| c.timestamp > datetime);

            Ok(())
        }

        // If the entry does not exist, we return an error
        dashmap::mapref::entry::Entry::Vacant(_) => {
            Err("No queue found for the given key".to_string())
        }
    }
}

pub async fn get_trends(candle: Arc<Candle>) -> Result<Option<DateTime<Utc>>, String> {
    let mut datetime: Option<DateTime<Utc>> = None;
    let key = format!("{}-{}", candle.symbol, candle.timerange);

    if let Some(trend) = TRENDS
        .get(key.as_str()) 
        .map(|t| Arc::clone(t.value()))
    {
        let mut modified_trend = (*trend).clone();

        // If the active trend is in the same direction as the candle
        if candle.direction == trend.direction {
            modified_trend.end_time = candle.timestamp;
            if let Some(subtrend) = SUBTRENDS
                .get(key.as_str())
                .map(|s| Arc::clone(s.value()))
            {
                // If a subtrend exists,
                // We check if we have to delete it
                // Or just update ti
                if subtrend.direction == "bullish" {
                    if candle.close < subtrend.low {
                        // In a bullish subtrend, (and a bearish trend + candle)
                        // If the candle closes below the last relative low,
                        // The subtrend is invalidated
                        // It's a BOS
                        // And we can update the trend

                        modified_trend.high = subtrend.high; 

                        let break_of_structure = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Break Of Structure".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: candle.timestamp,
                            price: subtrend.low,
                            // reference ?
                            direction: "Bearish".to_string(),
                        };

                        send_one_d_structure(&break_of_structure).await?;
                        if let Err(e) = add_one_d_structure(&break_of_structure) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }

                        let relative_high = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Relative High".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: subtrend.last_relative_high_datetime,
                            price: subtrend.high,
                            direction: "Relative High".to_string(),
                        };

                        let relative_low = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Relative Low".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: subtrend.last_relative_low_datetime,
                            price: subtrend.low,
                            direction: "Relative Low".to_string(),
                        };

                        send_one_d_structure(&relative_high).await?;
                        send_one_d_structure(&relative_low).await?;

                        if let Err(e) = add_one_d_structure(&relative_high) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }
                        if let Err(e) = add_one_d_structure(&relative_low) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }

                        // Remove the subtrend from the map
                        SUBTRENDS.remove(key.as_str());
                    } else {
                        let mut modified_subtrend = (*subtrend).clone();

                        // We just update the subtrend
                        if candle.high > subtrend.high {
                            modified_subtrend.high = candle.high;
                            modified_subtrend.last_relative_high = candle.high;
                            modified_subtrend.last_relative_high_datetime = candle.timestamp;
                        }

                        // Update the subtrend with the new candle
                        let new_subtrnd = Arc::new(modified_subtrend);
                        SUBTRENDS.insert(key.clone(), new_subtrnd);
                    }
                } else {
                    // So now if candle.direction is bearish
                    if candle.close > subtrend.high {
                        modified_trend.low = subtrend.low;

                        let break_of_structure = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Break Of Structure".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: candle.timestamp,
                            price: subtrend.high,
                            // reference ?
                            direction: "Bullish".to_string(),
                        };

                        send_one_d_structure(&break_of_structure).await?;
                        if let Err(e) = add_one_d_structure(&break_of_structure) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }

                        let relative_high = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Relative High".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: subtrend.last_relative_high_datetime,
                            price: subtrend.high,
                            direction: "Relative High".to_string()
                        };

                        let relative_low = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Relative Low".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: subtrend.last_relative_low_datetime,
                            price: subtrend.low,
                            direction: "Relative Low".to_string(),
                        };

                        send_one_d_structure(&relative_high).await?;
                        send_one_d_structure(&relative_low).await?;

                        if let Err(e) = add_one_d_structure(&relative_high) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }
                        if let Err(e) = add_one_d_structure(&relative_low) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }

                        // Remove the subtrend from the map
                        SUBTRENDS.remove(key.as_str());
                    } else {
                        let mut modified_subtrend = (*subtrend).clone();

                        // We just update the subtrend
                        if candle.low < subtrend.low {
                            modified_subtrend.low = candle.low;
                            modified_subtrend.last_relative_low = candle.low;
                            modified_subtrend.last_relative_low_datetime = candle.timestamp;
                        }

                        // Update the subtrend with the new candle
                        let new_subtrnd = Arc::new(modified_subtrend);
                        SUBTRENDS.insert(key.clone(), new_subtrnd);
                    }
                }
            } else {
                // Only update the trend
                if trend.low > candle.low {
                    modified_trend.low = candle.low;
                    modified_trend.low_datetime = Some(candle.timestamp);
                }

                if trend.high < candle.high {
                    modified_trend.high = candle.high; 
                    modified_trend.high_datetime = Some(candle.timestamp);
                }
            }
        } else {
            // If the candle is in the opposite direction of the active trend
            if let Some(subtrend) = SUBTRENDS
                .get(key.as_str())
                .map(|s| Arc::clone(s.value()))
            {
                if subtrend.direction == "bullish" {
                    modified_trend.end_time = candle.timestamp;
                    if candle.close > trend.high {
                        // If we are in a bearish trend
                        // And the candle closes above the last relative high of the trend
                        // This means that there is a reversal (Change Of Character)
                        
                        datetime = Some(subtrend.start_time);

                        let order_block = TwoDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Order Block".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: subtrend.last_candle.timestamp,
                            high: subtrend.last_candle.high,
                            low: subtrend.last_relative_low,
                            direction: "Bullish".to_string()
                        };

                        send_two_d_structure(&order_block).await?;

                        if let Err(e) = add_two_d_structure(&order_block) {
                            eprintln!("Failed to add 2D structure to the database: {}", e);
                        }

                        // TODO: change (not sure about it)
                        let change_of_character = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Change Of Character".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: candle.timestamp,
                            price: trend.relative_high.unwrap(),
                            direction: "Bullish".to_string(),
                            // Reference ?
                        };

                        send_one_d_structure(&change_of_character).await?;
                        if let Err(e) = add_one_d_structure(&change_of_character) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }
                    }
                } else if subtrend.direction == "bearish" {
                    modified_trend.end_time = candle.timestamp;

                    if candle.close < trend.low {
                        // If we are in a bullish trend
                        // And the candle closes below the last relative low of the trend
                        // This means that there is a reversal (Change Of Character)
                        
                        datetime = Some(subtrend.start_time);

                        let order_block = TwoDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Order Block".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: subtrend.last_candle.timestamp,
                            high: subtrend.last_candle.high,
                            low: subtrend.last_candle.low,
                            direction: "Bearish".to_string()
                        };

                        send_two_d_structure(&order_block).await?;
                        if let Err(e) = add_two_d_structure(&order_block) {
                            eprintln!("Failed to add 2D structure to the database: {}", e);
                        }

                        let change_of_character = OneDStructures {
                            symbol: candle.symbol.clone(),
                            structure: "Change Of Character".to_string(),
                            timerange: candle.timerange.clone(),
                            timestamp: candle.timestamp,
                            price: trend.relative_low.unwrap(),
                            direction: "Bearish".to_string(),
                            // Reference ?
                        };

                        send_one_d_structure(&change_of_character).await?;
                        if let Err(e) = add_one_d_structure(&change_of_character) {
                            eprintln!("Failed to add 1D structure to the database: {}", e);
                        }
                    }
                }
            } else {
                // If there is no subtrend, we create one
                let subtrend = Subtrend {
                    start_time: candle.timestamp,
                    direction: candle.direction.clone(),
                    high: candle.high,
                    low: candle.low,
                    last_relative_low: candle.low,
                    last_relative_high: candle.high,
                    last_candle: (*candle).clone(),
                    last_relative_low_datetime: candle.timestamp,
                    last_relative_high_datetime: candle.timestamp,
                };

                SUBTRENDS.insert(key.clone(), Arc::new(subtrend));
            }
        }

        // Update the trend and the subtrend with the new values
        let new_trend = Arc::new(modified_trend);
        TRENDS.insert(key.clone(), new_trend);        
    } else {
        // If there is not active trend
        // We create one from the actual candle
        // Only if the candle is not a doji
        if candle.direction == "doji" {
            return Ok(None);
        }

        let trend = Trend {
            symbol: candle.symbol.clone(),
            timerange: candle.timerange.clone(),
            start_time: candle.timestamp,
            end_time: candle.timestamp,
            direction: candle.direction.clone(),
            high: candle.high,
            low: candle.low,
            high_datetime: Some(candle.timestamp),
            low_datetime: Some(candle.timestamp),
            relative_high: Some(candle.high),
            relative_low: Some(candle.low),
        };

        TRENDS.insert(key.clone(), Arc::new(trend));
    }

    if let Some(_) = datetime {
        // If we have a datetime,
        // That means that we have an new trend, 
        // So we can send it and remove it from the map
        // Same for the subtrend because it's not longer needed
        let trend = TRENDS
            .get(key.as_str())
            .map(|t| Arc::clone(t.value()))
            .ok_or("No trend found for the given key")?;

        send_trend(&trend).await?;
        if let Err(e) = add_trend(&trend) {
            eprintln!("Failed to add trend to the database: {}", e);
        }

        TRENDS.remove(key.as_str());
        SUBTRENDS.remove(key.as_str());
    }

    Ok(datetime)
}