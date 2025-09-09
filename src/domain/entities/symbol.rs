use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum Symbol {
    EURUSD,
    Unknown,
}

impl Symbol {
    pub fn get_from_str(symbol: &str) -> Symbol {
        match symbol {
            "EURUSD" => Symbol::EURUSD,
            "EUR/USD" => Symbol::EURUSD,
            _ => Symbol::Unknown,
        }
    }

    pub fn into_text(&self) -> &'static str {
        match self {
            Symbol::EURUSD => "EUR/USD",
            Symbol::Unknown => "Unknown",
        }
    }
}
