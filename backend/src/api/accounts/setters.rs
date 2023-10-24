use crate::auth::password_reset::PasswordResetManager;
use crate::auth::permissions::Permissions;
use crate::auth::Authentication;
use crate::headers::Origin;
use crate::{DatabaseConnection, Error, Result, SharedConfig};

use actix_web::web::Data;
use actix_web::{put, web, HttpResponse};
use entities::account::AccountType;
use entities::account::ActiveModel;
use entities::emails::EmailType;
use entities::{emails, AccountEntity, AccountModel, ActiveAccountModel};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;
use utils::database::{EmailAddress, OptionalEmailAddress, Password};
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

#[put("/update/{user}/core")]
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

#[put("/update/{user}/active/{active}")]
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
#[put("/update/{user}/force-password-change")]
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
        debug!("Sending password reset email to {}", email);
        password.request(user.username, user.id, email, origin, true);
    }

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewAccount {
    pub name: String,
    pub username: String,
    #[serde(default)]
    pub description: String,
    pub quota: i64,
    #[serde(default)]
    pub require_password_change: bool,
    #[serde(default)]
    pub account_type: AccountType,
    pub backup_email: OptionalEmailAddress,
    pub group: i64,
    pub password: String,
    pub send_a_password_reset_email: bool,
    pub primary_email: OptionalEmailAddress,
}
#[derive(Deserialize)]
pub struct NewPassword {
    pub password: String,
}

#[put("/password/{user}")]
pub async fn set_password(
    auth: Authentication,
    data: web::Form<NewPassword>,
    user: web::Path<i64>,
    database: DatabaseConnection,
    settings: Data<SharedConfig>,
) -> Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let data = data.into_inner();
    let password = Password::new_hash(data.password, settings.password_hash)
        .map_err(|_| Error::UnableToHashPassword)?;

    let mut user: ActiveAccountModel = AccountEntity::find_by_id(user.into_inner())
        .one(database.as_ref())
        .await?
        .map(|x| x.into_active_model())
        .ok_or(Error::NotFound)?;

    user.password = ActiveValue::Set(password);

    user.save(database.as_ref()).await?;

    Ok(HttpResponse::NoContent().finish())
}
#[put("/new")]
pub async fn new(
    auth: Authentication,
    data: web::Json<NewAccount>,
    database: DatabaseConnection,
    settings: Data<SharedConfig>,
    password_reset: Data<PasswordResetManager>,

    origin: Origin,
) -> Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let data = data.into_inner();
    let password = Password::new_hash(data.password, settings.password_hash)
        .map_err(|_| Error::UnableToHashPassword)?;
    let user = ActiveModel {
        id: ActiveValue::NotSet,
        name: ActiveValue::Set(data.name),
        username: ActiveValue::Set(data.username),
        description: ActiveValue::Set(data.description),
        quota: ActiveValue::Set(data.quota),
        require_password_change: ActiveValue::Set(data.require_password_change),
        account_type: ActiveValue::Set(data.account_type),
        backup_email: ActiveValue::Set(data.backup_email.clone().0),
        active: Default::default(),
        group_id: ActiveValue::Set(data.group),
        created: Default::default(),
        password: ActiveValue::Set(password),
    };

    let result = AccountEntity::insert(user)
        .on_conflict(
            OnConflict::columns(vec![entities::account::Column::Username])
                .do_nothing()
                .to_owned(),
        )
        .exec(database.as_ref())
        .await;

    match result {
        Ok(ok) => {
            let id = ok.last_insert_id;
            if data.send_a_password_reset_email {
                let user: AccountModel = AccountEntity::find_by_id(id)
                    .one(database.as_ref())
                    .await?
                    .ok_or(Error::NotFound)?;
                if let Some(email) = data.backup_email.0 {
                    debug!("Sending password reset email to {}", email);
                    password_reset.request(
                        user.username,
                        user.id,
                        email,
                        origin,
                        user.require_password_change,
                    );
                } else {
                    debug!("No backup email provided, not sending password reset email");
                }
            }
            let primary_email_address_added = if let Some(value) = data.primary_email.0 {
                if emails::database_helper::does_primary_email_exist(
                    database.as_ref(),
                    value.clone(),
                )
                .await?
                {
                    false
                } else {
                    let new_email = entities::EmailActiveModel {
                        id: ActiveValue::NotSet,
                        account: ActiveValue::Set(id),
                        email_address: ActiveValue::Set(value),
                        email_type: ActiveValue::Set(EmailType::Primary),
                        created: Default::default(),
                    };
                    entities::EmailEntity::insert(new_email)
                        .exec(database.as_ref())
                        .await
                        .is_ok()
                }
            } else {
                false
            };

            Ok(HttpResponse::Created().json(json!({
                "id": id,
                "primary_email_address_added": primary_email_address_added
            })))
        }
        Err(_) => Ok(HttpResponse::Conflict().finish()),
    }
}
