use actix_web::error::ParseError;
use actix_web::http::header::ORIGIN;
use actix_web::{FromRequest, HttpMessage};
use std::ops::Deref;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Origin(pub String);

impl AsRef<str> for Origin {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for Origin {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Origin {
    type Error = ParseError;
    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        futures_util::future::ready(
            req.headers()
                .get(ORIGIN)
                .ok_or(ParseError::Header)
                .and_then(|value| {
                    info!("Bad Origin: {:?}", value);
                    value.to_str().map_err(|_err| ParseError::Header)
                })
                .map(|value| Origin(value.to_string())),
        )
    }
}
