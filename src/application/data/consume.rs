use crate::domain::entities::data::Data;
use crate::domain::ports::DataReceiver;

pub async fn consume_data<S: DataReceiver<Data> + ?Sized>(receiver: &S) {
    while let Some(_data) = receiver.receive_data().await {}
}
