use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Direction {
    Bullish,
    Bearish,
    Doji,
}
