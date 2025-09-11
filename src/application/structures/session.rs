use crate::application::context::AppContext;
use crate::domain::entities::candle::Candle;
use crate::domain::entities::data::Data;
use crate::domain::entities::session::{Session, SESSIONS};

use std::sync::Arc;

pub async fn process_session(ctx: &AppContext, candle: &Candle) {
    let key = candle.symbol;

    if let Some(mut session) = SESSIONS.get_mut(&key) {
        if session.contains(candle) {
            session.high = session.high.max(candle.high);
            session.low = session.low.min(candle.low);
            session.close = candle.close;
            session.volume += candle.volume;

            let session = Arc::new(Data::Session(session.clone()));
            ctx.send_data(session).await;

            return;
        }

        let session = Arc::new(Data::Session(session.clone()));
        ctx.insert_data(session).await;
    }
    // If there is no actual session in the same symbol
    // Or if we need to create a new session
    let session = Session::new(candle);

    SESSIONS.insert(key, session.clone());

    let session = Arc::new(Data::Session(session));
    ctx.send_data(session).await;
}
