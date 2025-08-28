use crate::domain::entities::candle::Candle;
use crate::domain::entities::session::{Session, SESSIONS};

pub async fn process_session(candle: &Candle) {
    let key = candle.symbol;

    if let Some(mut session) = SESSIONS.get_mut(&key) {
        if session.contains(candle) {
            session.high = session.high.max(candle.high);
            session.low = session.low.min(candle.low);
            session.close = candle.close;
            session.volume += candle.volume;

            // TODO: Send via websocket

            return;
        }

        // TODO: Add to the database
    }
    // If there is no actual session in the same symbol
    // Or if we need to create a new session
    let session = Session::new(candle);

    // TODO: Send via websocket

    SESSIONS.insert(key, session);
}
