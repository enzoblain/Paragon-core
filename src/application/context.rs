use crate::adapters::channel_adapter::ChannelAdapter;
use crate::domain::entities::data::Data;
use crate::domain::ports::DataSender;

use std::sync::Arc;

pub struct AppContext {
    pub data_inserter: Arc<dyn DataSender<Arc<Data>> + Send + Sync>,
    pub data_sender: Arc<dyn DataSender<Arc<Data>> + Send + Sync>,
}

impl AppContext {
    pub fn new(
        data_inserter: Arc<ChannelAdapter<Arc<Data>>>,
        data_sender: Arc<ChannelAdapter<Arc<Data>>>,
    ) -> Self {
        Self {
            data_inserter,
            data_sender,
        }
    }

    pub async fn insert_data(&self, data: Arc<Data>) {
        if let Err(e) = self.data_inserter.send_data(data).await {
            eprintln!("Error inserting data: {}", e);
        }
    }

    pub async fn send_data(&self, data: Arc<Data>) {
        if let Err(e) = self.data_sender.send_data(data).await {
            eprintln!("Error sending data: {}", e);
        }
    }
}
