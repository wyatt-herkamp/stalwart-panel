use actix_web::{
    get, put,
    web::{Data, ServiceConfig},
    HttpResponse,
};
use entities::account::panel_user::PanelUser;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel};
use utils::database::{EmailAddress, Password};

use crate::{auth::Authentication, DatabaseConnection, Error, SharedConfig};

pub fn init(service: &mut ServiceConfig) {
    service.service(me).service(change_password);
}
#[get("/me")]
pub async fn me(auth: Authentication) -> crate::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Into::<PanelUser>::into(auth)))
}
#[derive(serde::Deserialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}
#[put("/change-password")]
pub async fn change_password(
    auth: Authentication,
    body: actix_web::web::Form<ChangePassword>,
    database: DatabaseConnection,
    settings: Data<SharedConfig>,
) -> crate::Result<HttpResponse> {
    let user: PanelUser = auth.into();
    if !user
        .password
        .check_password(&body.old_password)
        .map_err(|_| Error::Unauthorized)?
    {
        return Err(Error::Unauthorized);
    }
    let mut user = user.into_active_model();
    user.password = ActiveValue::Set(
        Password::new_hash(&body.new_password, settings.password_hash)
            .map_err(|_| Error::BadRequest("Unable to Hash Password"))?,
    );
    user.save(database.as_ref()).await?;
    Ok(HttpResponse::NoContent().finish())
}
#[derive(serde::Deserialize)]
pub struct BackupEmail {
    pub backup_email: Option<EmailAddress>,
}
#[put("/backup-email")]
pub async fn backup_email(
    auth: Authentication,
    body: actix_web::web::Form<BackupEmail>,
    database: DatabaseConnection,
) -> crate::Result<HttpResponse> {
    let user: PanelUser = auth.into();
    let mut user = user.into_active_model();
    user.backup_email = ActiveValue::Set(body.backup_email.clone());
    user.save(database.as_ref()).await?;
    Ok(HttpResponse::NoContent().finish())
}
