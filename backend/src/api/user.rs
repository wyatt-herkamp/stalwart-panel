use crate::auth::Authentication;
use actix_web::web::ServiceConfig;
use actix_web::{get, HttpResponse};
use entities::account::panel_user::PanelUser;

pub fn init(service: &mut ServiceConfig) {
    service.service(me);
}
#[get("/me")]
pub async fn me(auth: Authentication) -> crate::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Into::<PanelUser>::into(auth)))
}
