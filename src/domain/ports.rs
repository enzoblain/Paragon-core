use crate::domain::entities::data::Data;

use std::future::Future;
use std::pin::Pin;

pub trait DataSender<T>: Sync {
    fn send_data(&self, data: T) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;
}

pub trait DataReceiver<T>: Sync {
    fn receive_data(&self) -> Pin<Box<dyn Future<Output = Option<T>> + Send + '_>>;
}

pub trait DataInserter {
    fn insert(&self, data: &Data) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;
}
