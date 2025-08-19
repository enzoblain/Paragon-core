use base64::{
    engine::general_purpose, 
    Engine as _
};
use rand::RngCore;
use chrono::NaiveTime;

pub fn is_in_timerange(start: NaiveTime, end: NaiveTime, time: NaiveTime) -> bool {
    if start <= end {
        time >= start && time <= end
    } else {
        // If the range wraps around midnight
        time >= start || time <= end
    }
}

pub fn generate_rand_base64() -> String {
    let mut key = [0u8; 16];

    rand::rng().fill_bytes(&mut key);
    
    general_purpose::STANDARD.encode(&key)
}