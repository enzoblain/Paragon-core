use crate::adapters::channel_adapter::ChannelAdapter;
use crate::adapters::rest_data_inserter::RestDataInserter;
use crate::domain::entities::data::Data;
use crate::domain::ports::{DataInserter, DataSender};

use std::sync::Arc;

pub struct AppContext {
    pub data_inserter: RestDataInserter,
    pub data_sender: Arc<dyn DataSender<Data> + Send + Sync>,
}

impl AppContext {
    pub fn new(data_inserter: RestDataInserter, data_sender: Arc<ChannelAdapter<Data>>) -> Self {
        Self {
            data_inserter,
            data_sender,
        }
    }

    pub async fn insert_data(&self, data: &Data) {
        if let Err(e) = self.data_inserter.insert(data).await {
            eprintln!("Error inserting data: {}", e);
        }
    }

    pub async fn send_data(&self, data: Data) {
        if let Err(e) = self.data_sender.send_data(data).await {
            eprintln!("Error sending data: {}", e);
        }
    }
}
