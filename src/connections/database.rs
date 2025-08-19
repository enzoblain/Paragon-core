use crate::{
    connections::auth::get_jwt, Candle, OneDStructures, Session, Trend, TwoDStructures
};
use common::{
    Config,
    entities::database::DatabaseData,
    utils::log::{
        LogFile,
        LogLevel
    }
};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

pub static DATABASEDATA: Lazy<Arc<DashMap<String, DatabaseData>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});

pub fn init_database_data(symbol: &str) {
    let data = DatabaseData {
        candles: Vec::new(),
        one_d_structure: Vec::new(),
        sessions: Vec::new(),
        trends: Vec::new(),
        two_d_structure: Vec::new(),
    };

    DATABASEDATA.insert(symbol.to_string(), data);
}

pub async fn send_data_to_database(symbol: &str) -> Result<(), String> {
    if let Some(data) = DATABASEDATA.get(symbol) {
        let client = Client::new();
        let query = r#"
            mutation Post($data: DatabaseData!) {
                post(data: $data)
            }
        "#;

        let data = data.clone();
        let variables = json!({ "data": data });

        let config = Config::global();
        let database_address = format!("{}:{}", config.server.database.address, config.server.database.port);

        let token = get_jwt("database".into()).map_err(|e| format!("Failed to get JWT: {}", e))?;

        LogFile::add_log(LogLevel::Info, &format!("Sending data to database for symbol: {}", symbol)).ok();

        let res = client.post(format!("http://{}/data", database_address))
            .bearer_auth(token)
            .json(&json!({
                "query": query,
                "variables": variables
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let json: Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if let Some(errors) = json.get("errors") {
            return Err(format!("GraphQL error: {}", errors));
        }

    } else {
        eprintln!("No data found for symbol: {}", symbol);
    }

    Ok(())
}

pub fn reset_database_data(symbol: &str) {
    DATABASEDATA.remove(symbol);
}

pub fn add_candle(candle: &Candle) -> Result<(), String> {
    let candle = candle.get_as_input();
    let symbol = candle.symbol.clone();

    if let Some(mut data) = DATABASEDATA.get_mut(symbol.as_str()) {
        data.candles.push(candle);
    } else {
        return Err("Symbol does not exist".into());
    }

    Ok(())
} 

pub fn add_one_d_structure(one_d_structure: &OneDStructures) -> Result<(), String> {
    let one_d_structure = one_d_structure.get_as_input();
    let symbol = one_d_structure.symbol.clone();

    if let Some(mut data) = DATABASEDATA.get_mut(symbol.as_str()) {
        data.one_d_structure.push(one_d_structure);
    } else {
        return Err("Symbol does not exist".into());
    }

    Ok(())
}

pub fn add_session(session: &Session) -> Result<(), String> {
    let session = session.get_as_input();
    let symbol = session.symbol.clone();

    if let Some(mut data) = DATABASEDATA.get_mut(symbol.as_str()) {
        data.sessions.push(session);
    } else {
        return Err("Symbol does not exist".into());
    }

    Ok(())
}

pub fn add_trend(trend: &Trend) -> Result<(), String> {
    let trend = trend.get_as_input();
    let symbol = trend.symbol.clone();

    if let Some(mut data) = DATABASEDATA.get_mut(symbol.as_str()) {
        data.trends.push(trend);
    } else {
        return Err("Symbol does not exist".into());
    }

    Ok(())
}

pub fn add_two_d_structure(two_d_structure: &TwoDStructures) -> Result<(), String> {
    let two_d_structure = two_d_structure.get_as_input();
    let symbol = two_d_structure.symbol.clone();

    if let Some(mut data) = DATABASEDATA.get_mut(symbol.as_str()) {
        data.two_d_structure.push(two_d_structure);
    } else {
        return Err("Symbol does not exist".into());
    }

    Ok(())
}