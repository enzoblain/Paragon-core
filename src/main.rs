use core::{
    connections::{
        database::{
            init_database_data,
            reset_database_data,
            send_data_to_database,
        },
        websocket::{
            create_channel,
            websocket_connection
        }},
    handlers::{
        candle::aggregate_candle,
        sessions::process_session
    }, 
    utils::temporary,
    TIMERANGES
};

use common::utils::log::{
    LogFile,
    LogLevel
};
use futures::future::join_all;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), String> {
    let rx = create_channel()?;

    let websocket_connection = tokio::spawn(async move {
        // Establish the WebSocket connection
        websocket_connection(rx).await
            .map_err(|e| format!("WebSocket connection error: {}", e))
    });

    let main_task = tokio::spawn(async move {   
        let data = temporary::get_data().map_err(|e| e.to_string())?;

        for index in 0..data.height() {
            init_database_data("EURUSD");

            let row = data.get_row(index).map_err(|e| e.to_string())?;
            let parsed_candle = temporary::parse_candle(row).map_err(|e| e.to_string())?;

            let candle = Arc::new(parsed_candle);

            let mut handles = Vec::new();

            // Spawn a task for each timerange to aggregate the candle
            for timerange in TIMERANGES.iter() {
                let cloned_candle = Arc::clone(&candle);
                let task = tokio::spawn(async move {
                    aggregate_candle(cloned_candle, "EURUSD", timerange).await
                });

                handles.push(task);
            }

            let cloned_candle = Arc::clone(&candle);
            let task = tokio::spawn(async move {
                if let Err(e) = process_session(cloned_candle, "EURUSD").await {
                    eprintln!("Error processing session: {}", e);
                }
            });

            handles.push(task);

            let _ = join_all(handles).await; 

            if let Err(e) = send_data_to_database("EURUSD").await {
                LogFile::add_log(LogLevel::Error, &format!("Error sending data to database: {}", e)).ok();
            } else {
                LogFile::add_log(LogLevel::Info, "Data sent to database successfully").ok();
            }

            reset_database_data("EURUSD");

            // Wait 5 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok::<(), String>(())
    });

    tokio::select! {
        res = websocket_connection => match res {
            Ok(Ok(())) => return Err("WebSocket connection finished without error but too early".into()),
            Ok(Err(e)) => return Err(format!("WebSocket connection error: {}", e)),
            Err(e) => return Err(format!("WebSocket connection panic : {}", e)),
        },
        res = main_task => match res {
            Ok(Ok(())) => return Err("Main task finished without error but too early".into()),
            Ok(Err(e)) => return Err(format!("Main task error: {}", e)),
            Err(e) => return Err(format!("Main panic : {}", e)),
        },
    }
}