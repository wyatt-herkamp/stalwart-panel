use crate::auth::session::SessionManager;
use crate::email_service::{EmailAccess, EmailType};
use crate::DatabaseConnection;
use crate::Result;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{get, post, web, HttpResponse};
use entities::account::panel_user::PanelUser;
use serde::Deserialize;
use crate::auth::password_reset::PasswordResetManager;

pub fn init(service: &mut ServiceConfig) {
    service
        .service(login)
        .service(request_password_reset)
        .service(verify_password_reset)
        .service(submit_password_reset);
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
#[post("/login")]
pub async fn login(
    post: web::Form<LoginRequest>,
    database: DatabaseConnection,
    session_manager: Data<SessionManager>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
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
) -> Result<HttpResponse> {
    let Some(panel_user) = PanelUser::get_by_backup_email(database.as_ref(), post.into_inner().backup_email)
        .await?else {
            return Ok(HttpResponse::NoContent().finish());
    };

    password_reset.request(panel_user);
    Ok(HttpResponse::NoContent().finish())
}

#[get("/reset/password/verify/{token}")]
pub async fn verify_password_reset(
    get: web::Path<String>,
    database: DatabaseConnection,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
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
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}
