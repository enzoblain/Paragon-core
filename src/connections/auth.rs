use server::utils::auth::create_jwt;

use jsonwebtoken::errors::{Error, ErrorKind};

pub fn get_jwt(to: String) -> Result<String, Error> {
    match to.as_str() {
        "database" => {
            return create_jwt("temp".into(), "admin".into(), None, Some("admin".into()), 5);
        }
        "websocket" => {
            return create_jwt("temp".into(), "sender".into(), Some("sender".into()), None, 5);
        }
        _ => {
            return Err(Error::from(ErrorKind::InvalidToken));
        }
    }
}