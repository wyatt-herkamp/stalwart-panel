use crate::auth::password_reset::PasswordResetManager;
use crate::auth::permissions::Permissions;
use crate::auth::Authentication;
use crate::headers::Origin;
use crate::{DatabaseConnection, Error, Result};

use actix_web::web::Data;
use actix_web::{put, web, HttpResponse};
use entities::account::AccountType;
use entities::{AccountEntity, AccountModel, ActiveAccountModel};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use utils::database::EmailAddress;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub description: Option<String>,
    pub quota: Option<i64>,
    pub account_type: Option<AccountType>,
    pub backup_email: Option<Option<EmailAddress>>,
}

impl UpdateAccount {
    pub fn apply_changes(self, user: &mut ActiveAccountModel) {
        if let Some(name) = self.name {
            user.name = ActiveValue::Set(name);
        }
        if let Some(description) = self.description {
            user.description = ActiveValue::Set(description);
        }
        if let Some(quota) = self.quota {
            user.quota = ActiveValue::Set(quota);
        }
        if let Some(account_type) = self.account_type {
            user.account_type = ActiveValue::Set(account_type);
        }
        if let Some(backup_email) = self.backup_email {
            user.backup_email = ActiveValue::Set(backup_email);
        }
    }
}

#[put("/account/update/{user}/core")]
pub async fn update_core(
    user: web::Path<i64>,
    auth: Authentication,
    data: web::Json<UpdateAccount>,
    database: DatabaseConnection,
) -> Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let mut user: ActiveAccountModel = AccountEntity::find_by_id(user.into_inner())
        .one(database.as_ref())
        .await?
        .map(|x| x.into_active_model())
        .ok_or(Error::NotFound)?;

    data.into_inner().apply_changes(&mut user);

    user.save(database.as_ref()).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[put("/account/update/{user}/active/{active}")]
pub async fn update_active(
    user: web::Path<(i64, bool)>,
    auth: Authentication,
    database: DatabaseConnection,
) -> Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let (user, active) = user.into_inner();
    let mut user: ActiveAccountModel = AccountEntity::find_by_id(user)
        .one(database.as_ref())
        .await?
        .map(|x| x.into_active_model())
        .ok_or(Error::NotFound)?;

    user.active = ActiveValue::Set(active);

    // TODO: Run post active hook

    user.save(database.as_ref()).await?;

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PasswordChange {
    pub send_email_to: Option<EmailAddress>,
}
#[put("/account/update/{user}/password-change")]
pub async fn password_change(
    user: web::Path<i64>,
    auth: Authentication,
    data: web::Json<PasswordChange>,
    database: DatabaseConnection,
    password: Data<PasswordResetManager>,
    origin: Origin,
) -> Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let mut user: ActiveAccountModel = AccountEntity::find_by_id(user.into_inner())
        .one(database.as_ref())
        .await?
        .map(|x| x.into_active_model())
        .ok_or(Error::NotFound)?;
    user.require_password_change = ActiveValue::Set(true);

    let user: AccountModel = user
        .save(database.as_ref())
        .await?
        .try_into_model()
        .unwrap();

    if let Some(email) = data.into_inner().send_email_to {
        password.request(user.username, user.id, email, origin, true);
    }

    Ok(HttpResponse::NoContent().finish())
}
