use crate::domain::entities::data::Data;
use crate::domain::ports::DataInserter;

use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct GraphQLDataInserter {
    graphql_url: String,
    client: Client,
}

impl GraphQLDataInserter {
    pub fn new(graphql_url: String) -> Self {
        Self {
            graphql_url,
            client: Client::new(),
        }
    }
}

impl DataInserter for GraphQLDataInserter {
    fn insert(&self, data: &Data) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
        let (mutation, variables) = match data {
            Data::Candle(candle) => (
                "mutation ($input: CandleInput!) { insertCandle(input: $input) }",
                json!({ "input": candle }),
            ),
            Data::OneDStructure(structure) => (
                "mutation ($input: OneDStructureInput!) { insertOneDStructure(input: $input) }",
                json!({ "input": structure }),
            ),
            Data::Trend(trend) => (
                "mutation ($input: TrendInput!) { insertTrend(input: $input) }",
                json!({ "input": trend }),
            ),
            Data::Session(session) => (
                "mutation ($input: SessionInput!) { insertSession(input: $input) }",
                json!({ "input": session }),
            ),
            Data::TwoDStructure(structure) => (
                "mutation ($input: TwoDStructureInput!) { insertTwoDStructure(input: $input) }",
                json!({ "input": structure }),
            ),
        };

        let body = json!({
            "query": mutation,
            "variables": variables,
        });

        let graphql_url = self.graphql_url.clone();
        let client = self.client.clone();
        let token = "temp";

        Box::pin(async move {
            let res = client
                .post(&graphql_url)
                .bearer_auth(token)
                .json(&body)
                .send()
                .await;

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
