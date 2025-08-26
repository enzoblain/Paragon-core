use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Candle {
    pub symbol: String,
    pub timerange: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub direction: String,
}

impl Candle {
    pub fn new(
        symbol: String,
        timerange: String,
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        let direction = if close > open {
            "bullish".to_string()
        } else if close < open {
            "bearish".to_string()
        } else {
            "doji".to_string()
        };

        Candle {
            symbol,
            timerange,
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            direction,
        }
    }
}