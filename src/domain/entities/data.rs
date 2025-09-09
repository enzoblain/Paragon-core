use crate::domain::entities::candle::Candle;
use crate::domain::entities::session::Session;
use crate::domain::entities::structures::{OneDStructure, TwoDStructure};
use crate::domain::entities::trend::Trend;

use serde_json::Value;

pub enum Data {
    Candle(Candle),
    OneDStructure(OneDStructure),
    Session(Session),
    Trend(Trend),
    TwoDStructure(TwoDStructure),
}

impl Data {
    pub fn into_request(&self) -> Value {
        match self {
            Data::Candle(candle) => candle.into_request(),
            Data::OneDStructure(structure) => structure.into_request(),
            Data::Session(session) => session.into_request(),
            Data::Trend(trend) => trend.into_request(),
            Data::TwoDStructure(structure) => structure.into_request(),
        }
    }
}
