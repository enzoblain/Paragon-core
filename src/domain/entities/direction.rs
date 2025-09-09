use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Direction {
    Bullish,
    Bearish,
    Doji,
}

impl Direction {
    pub fn into_text(self) -> &'static str {
        match self {
            Direction::Bullish => "bullish",
            Direction::Bearish => "bearish",
            Direction::Doji => "doji",
        }
    }
}
