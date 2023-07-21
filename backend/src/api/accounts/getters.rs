use crate::auth::permissions::Permissions;
use crate::auth::Authentication;
use crate::DatabaseConnection;

use actix_web::{get, web, HttpResponse};
use entities::account::database_helper::AccountSimple;
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

    let users = if query.active {
        AccountSimple::get_all_active_accounts(db.as_ref()).await?
    } else {
        AccountSimple::get_all_accounts(db.as_ref()).await?
    };

    Ok(HttpResponse::Ok().json(users))
}
