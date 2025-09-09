use crate::domain::entities::candle::Candle;
use crate::domain::entities::symbol::Symbol;
use crate::domain::entities::timerange::TIMERANGES;

use chrono::{DateTime, Utc};
use polars::{frame::row::Row, prelude::*};
use std::fs::File;

pub fn get_data() -> Result<DataFrame, PolarsError> {
    ParquetReader::new(File::open("data/EURUSD.parquet")?).finish()
}

pub fn parse_candle(row: Row) -> Result<Candle, String> {
    let datetime = if let AnyValue::Datetime(ts, TimeUnit::Microseconds, _) = row.0[0] {
        DateTime::<Utc>::from_naive_utc_and_offset(
            DateTime::from_timestamp_micros(ts)
                .expect("timestamp invalide")
                .naive_local(),
            Utc,
        )
    } else {
        Err("Invalid 'datetime' value in row: {row}")?
    };

    let open = if let AnyValue::Float64(val) = row.0[1] {
        val
    } else {
        Err("Invalid 'open' value in row: {row}")?
    };

    let high = if let AnyValue::Float64(val) = row.0[2] {
        val
    } else {
        Err("Invalid 'high' value in row: {row}")?
    };

    let low = if let AnyValue::Float64(val) = row.0[3] {
        val
    } else {
        Err("Invalid 'low' value in row: {row}")?
    };

    let close = if let AnyValue::Float64(val) = row.0[4] {
        val
    } else {
        Err("Invalid 'close' value in row: {row}")?
    };

    let volume = if let AnyValue::Int64(val) = row.0[5] {
        val
    } else {
        Err("Invalid 'volume' value in row: {row}")?
    };

    Ok(Candle::new(
        Symbol::EURUSD, // EUR/USD is the only symbol in the data
        &TIMERANGES[0], // We only add 1m candles for now
        datetime,
        open,
        high,
        low,
        close,
        volume as f64, // Convert i64 to f64 for volume
    ))
}
