use actix_web::ResponseError;
use either::Either;
use sea_orm::DbErr;
use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::auth::session::SessionError;
use this_actix_error::ActixError;
use thiserror::Error;

#[derive(Debug, Error, ActixError)]
pub enum WebsiteError {
    #[error("IO Error")]
    #[status_code(INTERNAL_SERVER_ERROR)]
    IoError(#[from] std::io::Error),
    #[error("Database Error")]
    DatabaseError(Either<DbErr, sqlx::Error>),
    #[error("Unauthorized")]
    #[status_code(UNAUTHORIZED)]
    Unauthorized,
    #[error("Bad Request: {0}")]
    #[status_code(BAD_REQUEST)]
    BadRequest(&'static str),
    #[error("Internal Error: {0}")]
    #[status_code(INTERNAL_SERVER_ERROR)]
    SessionError(#[from] SessionError),
}

/// Implemented for responses that can partially fail.
impl Serialize for WebsiteError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("WebsiteError", 2)?;
        s.serialize_field("status_code", &self.status_code().to_string())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

impl From<DbErr> for WebsiteError {
    fn from(error: DbErr) -> Self {
        Self::DatabaseError(Either::Left(error))
    }
}
impl From<sqlx::Error> for WebsiteError {
    fn from(error: sqlx::Error) -> Self {
        Self::DatabaseError(Either::Right(error))
    }
}
