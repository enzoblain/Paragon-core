use crate::domain::entities::direction::Direction;
use crate::domain::entities::symbol::Symbol;
use crate::domain::entities::timerange::Timerange;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize)]
pub enum OneDStructureLabel {
    BOS,
    CHOCH,
    RH,
    RL,
}

impl OneDStructureLabel {
    pub fn into_text(&self) -> &'static str {
        match self {
            OneDStructureLabel::BOS => "BoS",
            OneDStructureLabel::CHOCH => "CHoCH",
            OneDStructureLabel::RH => "RH",
            OneDStructureLabel::RL => "RL",
        }
    }
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

impl OneDStructure {
    pub fn into_request(&self) -> Value {
        json!({
            "symbol": self.symbol,
            "label": self.label.into_text(),
            "timerange": self.timerange.label,
            "timestamp": self.timestamp,
            "price": self.price,
            "direction": self.direction.into_text(),
        })
    }
}

#[derive(Serialize)]
pub enum TwoDStructureLabel {
    OB,
    FVG,
}

impl TwoDStructureLabel {
    pub fn into_text(&self) -> &'static str {
        match self {
            TwoDStructureLabel::OB => "OB",
            TwoDStructureLabel::FVG => "FVG",
        }
    }
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

impl TwoDStructure {
    pub fn into_request(&self) -> Value {
        json!({
            "symbol": self.symbol,
            "label": self.label.into_text(),
            "timerange": self.timerange.label,
            "timestamp": self.timestamp,
            "high": self.high,
            "low": self.low,
            "direction": self.direction.into_text(),
        })
    }
}
