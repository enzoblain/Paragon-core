use crate::domain::entities::direction::Direction;
use crate::domain::entities::symbol::Symbol;
use crate::domain::entities::timerange::Timerange;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub enum OneDStructureLabel {
    BOS,
    CHOCH,
    RH,
    RL,
}

#[derive(Serialize)]
pub struct OneDStructure {
    pub symbol: Symbol,
    pub label: OneDStructureLabel,
    pub timerange: &'static Timerange,
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub direction: Direction,
}

#[derive(Serialize)]
pub enum TwoDStructureLabel {
    OB,
    FVG,
}

#[derive(Serialize)]
pub struct TwoDStructure {
    pub symbol: Symbol,
    pub label: TwoDStructureLabel,
    pub timerange: &'static Timerange,
    pub timestamp: DateTime<Utc>,
    pub high: f64,
    pub low: f64,
    pub direction: Direction,
}
