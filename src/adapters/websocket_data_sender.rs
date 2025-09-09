use base64::{engine::general_purpose, Engine as _};
use http::{header::AUTHORIZATION, Request};
use rand::{rng, RngCore};

pub struct WebsocketDataSender {
    pub url: String,
    pub token: String,
    pub req: Request<()>,
}

impl WebsocketDataSender {
    pub fn new(url: String) -> Self {
        let token = if let Ok(t) = std::env::var("WS_TOKEN") {
            t
        } else {
            eprintln!("Warning: API_TOKEN not set. Using empty token.");
            String::new()
        };

        let req = Request::builder()
            .uri(url.as_str())
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header("sec-websocket-key", generate_rand_base64())
            .header("host", &url)
            .header("connection", "Upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .body(())
            .map_err(|e| format!("Failed to build request: {}", e))
            .unwrap();

        Self { url, token, req }
    }
}

pub fn generate_rand_base64() -> String {
    let mut key = [0u8; 16];

    rng().fill_bytes(&mut key);

    general_purpose::STANDARD.encode(key)
}
