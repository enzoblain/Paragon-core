use crate::domain::entities::timerange::Timerange;

use chrono::{DateTime, Duration, TimeZone, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Candle {
    pub symbol: String,
    pub timerange: &'static Timerange,
    pub timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Bullish,
    Bearish,
    Doji,
}

impl Candle {
    pub fn new(
        symbol: String,
        timerange: &'static Timerange,
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        let direction = Self::compute_direction(open, close);
        let (timestamp, end_timestamp) = Self::align_timestamps(timestamp, timerange.duration_ms);

        Candle {
            symbol,
            timerange,
            end_timestamp,
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            direction,
        }
    }

    fn align_timestamps(timestamp: DateTime<Utc>, duration_ms: i64) -> (DateTime<Utc>, DateTime<Utc>) {
        let aligned = Utc.timestamp_millis_opt(
            (timestamp.timestamp_millis() / duration_ms) * duration_ms
        ).single().unwrap();
        let end = aligned + Duration::milliseconds(duration_ms);

        (aligned, end)
    }

    fn compute_direction(open: f64, close: f64) -> Direction {
        match close.partial_cmp(&open) {
            Some(Ordering::Greater) => Direction::Bullish,
            Some(Ordering::Less) => Direction::Bearish,
            _ => Direction::Doji,
        }
    }
}

// Stores the current state of all candles.
// Enables modification of candles when new data is received for an ongoing time range.
pub static CANDLES: Lazy<DashMap<String, Candle>> = Lazy::new(|| {
    DashMap::new()
});
