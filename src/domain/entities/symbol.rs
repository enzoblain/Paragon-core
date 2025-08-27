#[derive(Clone, Copy, Debug, Eq,Hash, PartialEq)]
pub enum Symbol {
    EURUSD,
    Unkwown
}

impl Symbol {
    pub fn get_from_str(
        symbol: &str
    ) -> Symbol {
        match symbol {
            "EURUSD" => Symbol::EURUSD,
            "EUR/USD" => Symbol::EURUSD,
            _ => Symbol::Unkwown
        }
    }
}