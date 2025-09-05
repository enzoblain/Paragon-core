use crate::domain::ports::{DataReceiver, DataSender};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct ChannelAdapter<T> {
    tx: mpsc::Sender<T>,
    rx: Mutex<mpsc::Receiver<T>>,
}

impl<T> ChannelAdapter<T> {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);

        ChannelAdapter {
            tx,
            rx: Mutex::new(rx),
        }
    }
}

impl<T: Send + 'static> DataSender<T> for ChannelAdapter<T> {
    fn send_data(&self, data: T) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
        let tx = self.tx.clone();

        Box::pin(async move { tx.send(data).await.map_err(|e| e.to_string()) })
    }
}

impl<T: Send + 'static> DataSender<T> for Arc<ChannelAdapter<T>> {
    fn send_data(&self, data: T) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
        (**self).send_data(data)
    }
}

impl<T: Send + 'static> DataReceiver<T> for ChannelAdapter<T> {
    fn receive_data(&self) -> Pin<Box<dyn Future<Output = Option<T>> + Send + '_>> {
        Box::pin(async move {
            let mut rx_guard = self.rx.lock().await;
            rx_guard.recv().await
        })
    }
}

impl<T: Send + 'static> DataReceiver<T> for Arc<ChannelAdapter<T>> {
    fn receive_data(&self) -> Pin<Box<dyn Future<Output = Option<T>> + Send + '_>> {
        (**self).receive_data()
    }
}
