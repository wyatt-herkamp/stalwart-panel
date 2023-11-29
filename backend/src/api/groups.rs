use actix_web::{get, web::ServiceConfig, HttpResponse};
use entities::GroupEntity;
use sea_orm::prelude::*;

use crate::auth::permissions::Permissions;
pub fn init(service: &mut ServiceConfig) {
    service.service(get_groups);
}

#[get("/list")]
pub async fn get_groups(
    database: crate::DatabaseConnection,
    auth: crate::auth::Authentication,
) -> crate::Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let groups = GroupEntity::find().all(database.as_ref()).await?;
    Ok(HttpResponse::Ok().json(groups))
}
