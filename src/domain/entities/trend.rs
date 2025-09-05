use crate::domain::entities::candle::Candle;
use crate::domain::entities::direction::Direction;
use crate::domain::entities::symbol::Symbol;
use crate::domain::entities::timerange::Timerange;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Serialize;

pub static QUEUE: Lazy<DashMap<(Symbol, &'static Timerange), Vec<Candle>>> =
    Lazy::new(DashMap::new);
pub static TRENDS: Lazy<DashMap<(Symbol, &'static Timerange), Trend>> = Lazy::new(DashMap::new);
pub static SUBTRENDS: Lazy<DashMap<(Symbol, &'static Timerange), Subtrend>> =
    Lazy::new(DashMap::new);

#[derive(Clone, Serialize)]
pub struct Trend {
    pub symbol: Symbol,
    pub timerange: Timerange,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub direction: Direction,
    pub high: f64,
    pub low: f64,
    pub high_datetime: Option<DateTime<Utc>>,
    pub low_datetime: Option<DateTime<Utc>>,
    pub relative_high: Option<f64>,
    pub relative_low: Option<f64>,
}

pub struct Subtrend {
    pub start_time: DateTime<Utc>,
    pub direction: Direction,
    pub high: f64,
    pub low: f64,
    pub last_relative_low: f64,
    pub last_relative_high: f64,
    pub last_candle: Candle,
    pub last_relative_low_datetime: DateTime<Utc>,
    pub last_relative_high_datetime: DateTime<Utc>,
}
