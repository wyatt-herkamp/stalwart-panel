use actix_web::{
    cookie::{CookieBuilder, Expiration, SameSite},
    get, post, web,
    web::{Data, ServiceConfig},
    HttpRequest, HttpResponse,
};
use chrono::Duration;
use entities::{account::panel_user::PanelUser, AccountEntity};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};
use tracing::warn;
use utils::database::Password;

use crate::{
    auth::{
        password_reset::PasswordResetManager,
        session::{Session, SessionManager},
    },
    headers::Origin,
    DatabaseConnection, Error, Result, SharedConfig,
};

pub fn init(service: &mut ServiceConfig) {
    service
        .service(login)
        .service(request_password_reset)
        .service(verify_password_reset)
        .service(submit_password_reset)
        .service(logout);
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
#[derive(Serialize)]
pub struct LoginResponse {
    panel_user: PanelUser,
    session: Session,
}
#[get("/logout")]
pub async fn logout(
    session_manager: Data<SessionManager>,
    http_request: HttpRequest,
) -> Result<HttpResponse> {
    if let Some(session_id) = http_request
        .cookie("session")
        .map(|v| v.value().to_string())
    {
        session_manager.as_ref().delete_session(&session_id)?;
    }

    Ok(HttpResponse::NoContent().finish())
}
#[post("/login")]
pub async fn login(
    post: web::Form<LoginRequest>,
    database: DatabaseConnection,
    session_manager: Data<SessionManager>,
) -> Result<HttpResponse> {
    let post = post.into_inner();
    let panel_user = PanelUser::get(database.as_ref(), &post.username)
        .await?
        .ok_or(Error::Unauthorized)?;

    if panel_user
        .password
        .check_password(post.password)
        .map_err(|e| {
            warn!("Failed to check password: {}", e);
            Error::Unauthorized
        })?
    {
        return Err(Error::Unauthorized);
    }
    let duration = Duration::days(1);

    let session = session_manager.create_session(panel_user.id, duration)?;

    let new_cookie = CookieBuilder::new("session", session.session_id.clone())
        .path("/")
        .secure(true)
        .same_site(SameSite::None)
        .expires(Expiration::Session)
        .finish();

    Ok(HttpResponse::Ok().cookie(new_cookie).json(LoginResponse {
        panel_user,
        session,
    }))
}

#[derive(Deserialize)]
pub struct PasswordReset {
    pub backup_email: String,
}
#[post("/reset/password/request")]
pub async fn request_password_reset(
    post: web::Form<PasswordReset>,
    database: DatabaseConnection,
    password_reset: Data<PasswordResetManager>,
    origin: Origin,
) -> Result<HttpResponse> {
    let Some(panel_user) =
        PanelUser::get_by_backup_email(database.as_ref(), post.into_inner().backup_email).await?
    else {
        return Ok(HttpResponse::NoContent().finish());
    };

    password_reset.request(
        panel_user.username,
        panel_user.id,
        panel_user.backup_email.unwrap(),
        origin,
        false,
    );

    Ok(HttpResponse::NoContent().finish())
}

#[get("/reset/password/verify/{token}")]
pub async fn verify_password_reset(
    get: web::Path<String>,
    password_reset: Data<PasswordResetManager>,
) -> HttpResponse {
    if password_reset.get_request(get.as_ref()).is_some() {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[derive(Deserialize)]
pub struct PasswordResetSubmit {
    pub password: String,
}

#[get("/reset/password/submit/{token}")]
pub async fn submit_password_reset(
    get: web::Path<String>,
    post: web::Form<PasswordResetSubmit>,
    database: DatabaseConnection,
    password_reset: Data<PasswordResetManager>,
    shared_settings: Data<SharedConfig>,
) -> Result<HttpResponse> {
    if let Some(value) = password_reset.get_request(get.as_ref()) {
        let password = post.into_inner().password;

        let Some(mut user_model) = AccountEntity::find_by_id(value.account_id)
            .one(database.as_ref())
            .await?
            .map(|v| v.into_active_model())
        else {
            warn!("Failed to find account with id {}", value.account_id);
            return Ok(HttpResponse::NotFound().finish());
        };
        user_model.require_password_change = ActiveValue::set(false);
        user_model.password = ActiveValue::set(
            Password::new_hash(password, shared_settings.password_hash).map_err(|e| {
                warn!("Failed to hash password: {}", e);
                Error::BadRequest("Failed to hash password")
            })?,
        );

        drop(value);
        password_reset.remove_request(get.as_ref());
        AccountEntity::update(user_model)
            .exec(database.as_ref())
            .await?;
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
