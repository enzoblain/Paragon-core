use crate::domain::entities::data::Data;
use crate::domain::ports::DataInserter;

use reqwest::Client;
use serde_json::Value;
use std::env;
use std::future::Future;
use std::pin::Pin;

pub struct RestDataInserter {
    url: String,
    client: Client,
    token: String,
}

impl RestDataInserter {
    pub fn new(url: String) -> Self {
        let token = if let Ok(t) = env::var("API_TOKEN") {
            t
        } else {
            eprintln!("Warning: API_TOKEN not set. Using empty token.");
            String::new()
        };

        Self {
            url,
            client: Client::new(),
            token,
        }
    }
}

impl DataInserter for RestDataInserter {
    fn insert<'a>(
        &'a self,
        data: &Data,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
        let body = data.into_request();

        let client = &self.client;
        let url = &self.url;
        let token = &self.token;

        Box::pin(async move {
            let res = client.post(url).bearer_auth(token).json(&body).send().await;

            match res {
                Ok(response) if response.status().is_success() => {
                    let json: Value = response
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

                    if json.get("errors").is_some() {
                        Err(format!("GraphQL error: {}", json["errors"]))
                    } else {
                        Ok(())
                    }
                }
                Ok(response) => Err(format!("Failed to insert data: {}", response.status())),
                Err(err) => Err(format!("Error sending request: {}", err)),
            }
        })
    }
}
