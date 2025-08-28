use crate::domain::entities::candle::Candle;
use crate::domain::entities::symbol::Symbol;
use crate::domain::entities::timerange::Timerange;

use dashmap::DashMap;
use once_cell::sync::Lazy;

// For the fair value gaps
pub static LAST_THREE_CANDLES: Lazy<DashMap<(Symbol, &'static Timerange), Vec<Candle>>> = Lazy::new(|| {
    DashMap::new()
});