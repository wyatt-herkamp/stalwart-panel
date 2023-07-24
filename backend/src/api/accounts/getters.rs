use crate::auth::permissions::Permissions;
use crate::auth::Authentication;
use crate::DatabaseConnection;

use actix_web::{get, web, HttpResponse};
use entities::account::database_helper::AccountSimple;
use entities::account::full_user::FullUser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct List {
    pub active: bool,
}

#[get("/list")]
pub async fn list(
    auth: Authentication,
    query: web::Query<List>,
    db: DatabaseConnection,
) -> crate::Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }
    if query.active {
        AccountSimple::get_all_active_accounts(db.as_ref()).await
    } else {
        AccountSimple::get_all_accounts(db.as_ref()).await
    }
    .map(|users| HttpResponse::Ok().json(users))
    .map_err(Into::into)
}

#[derive(Debug, Deserialize)]
pub struct GetUser {
    include_emails: bool,
}

#[get("/get/{user}")]
pub async fn get_full_user(
    auth: Authentication,
    user: web::Path<i64>,
    get_params: web::Query<GetUser>,
    db: DatabaseConnection,
) -> crate::Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    FullUser::get_by_id(db.as_ref(), user.into_inner(), get_params.include_emails)
        .await?
        .map(|user| HttpResponse::Ok().json(user))
        .ok_or(crate::Error::NotFound)
}
