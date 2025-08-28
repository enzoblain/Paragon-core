use crate::domain::entities::candle::Candle;
use crate::domain::entities::symbol::Symbol;

use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, Timelike, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;

pub struct Session {
    pub symbol: Symbol,
    pub label: RefSessions,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}

impl Session {
    pub fn contains(&self, candle: &Candle) -> bool {
        candle.timestamp >= self.start_time && candle.timestamp < self.end_time
    }

    pub fn new(candle: &Candle) -> Self {
        let ref_sessions = RefSessions::from_timestamp(candle.timestamp.time()).unwrap();
        let (start_time, end_time) =
            RefSessions::get_start_end_datetime(&ref_sessions, candle.timestamp);

        Self {
            symbol: candle.symbol,
            label: ref_sessions,
            start_time,
            end_time,
            high: candle.high,
            low: candle.low,
            open: candle.open,
            close: candle.close,
            volume: candle.volume,
        }
    }
}

pub struct RefSession {
    pub label: &'static str,
    pub start: NaiveTime,
    pub end: NaiveTime,
}

// All the sessions are in UTC time
pub static REFSESSIONS: &[RefSession] = &[
    RefSession {
        label: "Asian Session",
        start: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
    },
    RefSession {
        label: "London Session",
        start: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
        end: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    },
    RefSession {
        label: "New York Session",
        start: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
    },
];

pub enum RefSessions {
    Asian,
    London,
    NewYork,
}

impl RefSessions {
    pub fn get_start_end_datetime(
        ref_session: &RefSessions,
        timestamp: DateTime<Utc>,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let mut start_date = timestamp.date_naive();
        let mut end_date = timestamp.date_naive();

        // Overnight session adjustment
        if std::ptr::eq(ref_session, &RefSessions::Asian) {
            if timestamp.hour() < REFSESSIONS[0].start.hour()
                || timestamp.hour() == REFSESSIONS[0].start.hour()
                    && timestamp.minute() < REFSESSIONS[0].start.minute()
            {
                start_date -= -Duration::days(1);
            } else {
                end_date += Duration::days(1);
            }
        }

        let (start_time, end_time) = Self::get_start_end_time(ref_session);

        let start = NaiveDateTime::new(start_date, start_time);
        let end = NaiveDateTime::new(end_date, end_time);

        (start.and_utc(), end.and_utc())
    }

    pub fn get_start_end_time(ref_session: &RefSessions) -> (NaiveTime, NaiveTime) {
        match ref_session {
            RefSessions::Asian => (REFSESSIONS[0].start, REFSESSIONS[0].end),
            RefSessions::London => (REFSESSIONS[1].start, REFSESSIONS[1].end),
            RefSessions::NewYork => (REFSESSIONS[2].start, REFSESSIONS[2].end),
        }
    }

    pub fn from_session(session: &RefSession) -> Option<Self> {
        if std::ptr::eq(session, &REFSESSIONS[0]) {
            Some(RefSessions::Asian)
        } else if std::ptr::eq(session, &REFSESSIONS[1]) {
            Some(RefSessions::London)
        } else if std::ptr::eq(session, &REFSESSIONS[2]) {
            Some(RefSessions::NewYork)
        } else {
            None
        }
    }

    pub fn from_timestamp(timestamp: NaiveTime) -> Option<Self> {
        for session in REFSESSIONS {
            if session.start <= session.end {
                if timestamp >= session.start && timestamp < session.end {
                    return Self::from_session(session);
                }
            } else {
                // Overnight session
                if timestamp >= session.start || timestamp < session.end {
                    return Self::from_session(session);
                }
            }
        }
        None
    }
}

pub static SESSIONS: Lazy<DashMap<Symbol, Session>> = Lazy::new(DashMap::new);
