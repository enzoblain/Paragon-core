use crate::{
    Candle, 
    connections::database::add_session,
    ReferenceSession, 
    Session, 
    SESSIONS,
    utils::utils::is_in_timerange
};

use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static SESSION: Lazy<Arc<DashMap<String, Session>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});

pub async fn process_session(candle: Arc<Candle>, symbol: &'static str) -> Result<(), String> {
    let key = format!("{}-{}", symbol, candle.timerange);

    let should_create_new_session = should_create_new_session(candle.clone()).await;

    if should_create_new_session {
        let session = get_right_session(candle.timestamp)?;

        let (start, end) = get_sessions_start_end(candle.timestamp, session);

        let new_session = Session {
            symbol: symbol.to_string(),
            label: session.label.to_string(),
            start_time: start,
            end_time: end,
            high: candle.high,
            low: candle.low,
            open: candle.open,
            close: candle.close,
            volume: candle.volume,
        };

        SESSION.entry(key).and_modify(|e| {
            *e = new_session.clone();
        }).or_insert(new_session.clone());

    } else {
        let mut current_session = SESSION.get_mut(&key).ok_or_else(|| {
            format!("Session not found for key: {}", key)
        })?;

        if candle.high > current_session.high {
            current_session.high = candle.high;
        }
        if candle.low < current_session.low {
            current_session.low = candle.low;
        }

        current_session.close = candle.close;
        current_session.volume += candle.volume;
    }

    Ok(())
}

pub async fn should_create_new_session(candle: Arc<Candle>) -> bool {
    let session = SESSION
        .get(&format!("{}-{}", candle.symbol, candle.timerange))
        .map(|e| e.value().clone());

    if session.is_none() {
        return true;
    }

    // If the candle is not in the current session (its timestamp is not in the session's start and end)
    if let Some(session) = session.as_ref() {
        if !is_same_session(session, candle) {
            if let Err(e) = add_session(session) {
                eprintln!("Failed to add session to the database: {}", e);
            }

            return true;
        }
    }

    return false;
}

pub fn is_same_session(session: &Session, candle: Arc<Candle>) -> bool {
    candle.timestamp >= session.start_time && candle.timestamp <= session.end_time
}

pub fn get_right_session(timestamp: DateTime<Utc>) -> Result<&'static ReferenceSession, String> {
    for session in SESSIONS.iter() {
        if is_in_timerange(session.start, session.end, timestamp.time()) {
            return Ok(session);
        }
    }
    
    Err(format!("No session found for timestamp: {}", timestamp))
}

pub fn get_sessions_start_end(timestamp: DateTime<Utc>, session: &ReferenceSession) -> (DateTime<Utc>, DateTime<Utc>) {
    let mut start_date = timestamp.date_naive();
    let mut end_date = timestamp.date_naive();

    // Because the asian session crosses the day
    // We need to check which day we are in
    if session.label == "Asian Session" {
        if timestamp.hour() < 7 {
            start_date = start_date.pred_opt().unwrap();
        } else { // Else the session end the next day
            end_date = end_date.succ_opt().unwrap();
        }
    }

    let start = NaiveDateTime::new(
        start_date,
        session.start
    );
    let end = NaiveDateTime::new(
        end_date,
        session.end
    );

    (start.and_utc(), end.and_utc())
}