use crate::domain::entities::candle::Candle;
use crate::domain::entities::session::Session;
use crate::domain::entities::structures::{OneDStructure, TwoDStructure};
use crate::domain::entities::trend::Trend;

pub enum Data {
    Candle(Candle),
    OneDStructure(OneDStructure),
    Session(Session),
    Trend(Trend),
    TwoDStructure(TwoDStructure),
}
