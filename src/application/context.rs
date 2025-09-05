use crate::adapters::channel_adapter::ChannelAdapter;
use crate::adapters::graphql_data_inserter::GraphQLDataInserter;
use crate::domain::entities::data::Data;
use crate::domain::ports::{DataInserter, DataSender};

use std::sync::Arc;

pub struct AppContext {
    pub data_inserter: GraphQLDataInserter,
    pub data_sender: Arc<dyn DataSender<Data> + Send + Sync>,
}

impl AppContext {
    pub fn new(graphql_url: String, data_sender: Arc<ChannelAdapter<Data>>) -> Self {
        let data_inserter = GraphQLDataInserter::new(graphql_url);
        Self {
            data_inserter,
            data_sender,
        }
    }

    pub async fn insert_data(&self, data: &Data) -> Result<(), String> {
        self.data_inserter.insert(data).await
    }

    pub async fn send_data(&self, data: Data) -> Result<(), String> {
        self.data_sender.send_data(data).await
    }
}
