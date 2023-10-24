use crate::{Error, SharedConfig};
use actix_web::http::header::{HOST, ORIGIN};
use actix_web::web::Data;
use actix_web::FromRequest;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Deref;
use tracing::debug;

#[derive(Debug, Clone, Serialize)]
pub struct Origin {
    pub url: String,
    pub is_https: bool,
}
impl Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.url)
    }
}
impl AsRef<str> for Origin {
    fn as_ref(&self) -> &str {
        &self.url
    }
}
impl Into<String> for Origin {
    fn into(self) -> String {
        self.url
    }
}
impl Deref for Origin {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl Origin {
    #[inline]
    fn from_request_inner(req: &actix_web::HttpRequest) -> Result<Self, Error> {
        let (url, https) = if let Some(value) = req.headers().get(ORIGIN) {
            value
                .to_str()
                .map(|value| (value.to_string(), value.starts_with("https")))?
        } else if let Some(value) = req.headers().get(HOST) {
            let https = req
                .app_data::<Data<SharedConfig>>()
                .map(|v| v.as_ref().https)
                .unwrap_or_default();
            let value = value.to_str()?;
            let url = if https {
                format!("https://{}", value)
            } else {
                format!("http://{}", value)
            };
            (url, https)
        } else {
            debug!("No Host or Origin header found");
            return Err(Error::BadRequest("No Host or Origin header found"));
        };

        Ok(Origin {
            url,
            is_https: https,
        })
    }
}
impl FromRequest for Origin {
    type Error = Error;
    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = Origin::from_request_inner(req);
        futures_util::future::ready(result)
    }
}
