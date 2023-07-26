use crate::auth::permissions::Permissions;
use crate::auth::Authentication;
use crate::error::WebsiteError;
use crate::{DatabaseConnection, Result};
use actix_web::web::ServiceConfig;
use actix_web::{delete, put, web, HttpResponse};
use entities::emails::{EmailType, Emails};
use entities::{AccountEntity, EmailActiveModel, EmailEntity, EmailModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DeleteResult, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, TryIntoModel,
};
use serde::Deserialize;
use tracing::debug;
use utils::database::EmailAddress;
pub fn init(service: &mut ServiceConfig) {
    service.service(add_email).service(delete_email);
}

#[derive(Debug, Deserialize)]
pub struct NewEmail {
    pub email: EmailAddress,
    pub email_type: EmailType,
}

#[put("/{user}")]
pub async fn add_email(
    connection: DatabaseConnection,
    account_id: web::Path<i64>,
    email: web::Json<NewEmail>,
    auth: Authentication,
) -> Result<HttpResponse> {
    if !auth.can_manage_users() {
        return Err(WebsiteError::Unauthorized);
    }
    let user = account_id.into_inner();
    if AccountEntity::find_by_id(user)
        .count(connection.as_ref())
        .await?
        == 0
    {
        return Err(WebsiteError::NotFound);
    }

    let NewEmail {
        email: email_address,
        email_type,
    } = email.into_inner();

    let emails = Emails::get_by_user_id(connection.as_ref(), user).await?;

    let email = if let Some(value) = emails.get_by_address(&email_address) {
        if value.email_type == email_type {
            return Ok(HttpResponse::Conflict().json(value));
        } else {
            value.clone().into_active_model()
        }
    } else {
        EmailActiveModel {
            id: Default::default(),
            account: ActiveValue::Set(user),
            email_address: ActiveValue::Set(email_address),
            email_type: ActiveValue::Set(email_type),
            created: entities::now(),
        }
    };

    if email_type == EmailType::Primary {
        if let Some(value) = emails.get_primary().cloned() {
            debug!("If you put a primary email on an account that already has a primary email, the old primary email will be converted to an alias");
            let mut value = value.into_active_model();
            value.email_type = ActiveValue::Set(EmailType::Alias);
            value.save(connection.as_ref()).await?;
        }
    }

    let active: EmailModel = email.save(connection.as_ref()).await?.try_into_model()?;

    Ok(HttpResponse::Ok().json(active))
}
#[derive(Debug, Deserialize)]
pub struct EmailAddressRemove {
    pub email_address: EmailAddress,
    #[serde(default)]
    pub purge_emails_to_address_in_account: bool,
}
#[delete("/{user}")]
pub async fn delete_email(
    connection: DatabaseConnection,
    account_id: web::Path<i64>,
    email_address: web::Form<EmailAddressRemove>,
    auth: Authentication,
) -> Result<HttpResponse> {
    use entities::emails::Column as EmailColumn;
    if !auth.can_manage_users() {
        return Err(WebsiteError::Unauthorized);
    }

    let user = account_id.into_inner();
    let email_address = email_address.into_inner().email_address;
    let result: DeleteResult = EmailEntity::delete_many()
        .filter(
            EmailColumn::EmailAddress
                .eq(email_address)
                .and(EmailColumn::Account.eq(user)),
        )
        .exec(connection.as_ref())
        .await?;

    return if result.rows_affected == 0 {
        Err(WebsiteError::NotFound)
    } else {
        // TODO purge emails in the account that are now orphaned-
        Ok(HttpResponse::NoContent().finish())
    };
}
