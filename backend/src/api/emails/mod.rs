use actix_web::{delete, put, web, web::ServiceConfig, HttpResponse};
use entities::{
    emails,
    emails::{ActiveModel, EmailType},
    AccountEntity, EmailActiveModel, EmailEntity,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DeleteResult, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, TryIntoModel,
};
use serde::Deserialize;
use tracing::debug;
use utils::database::EmailAddress;

use crate::{
    auth::{permissions::Permissions, Authentication},
    error::WebsiteError,
    DatabaseConnection, Result,
};
pub fn init(service: &mut ServiceConfig) {
    service.service(add_or_update).service(delete_email);
}

#[derive(Debug, Deserialize)]
pub struct AddOrUpdateEmail {
    pub id: Option<i64>,
    pub email_address: EmailAddress,
    pub email_type: EmailType,
}

#[put("/{user}")]
pub async fn add_or_update(
    connection: DatabaseConnection,
    account_id: web::Path<i64>,
    email: web::Json<AddOrUpdateEmail>,
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

    let AddOrUpdateEmail {
        id,
        email_address,
        email_type,
    } = email.into_inner();

    let email: ActiveModel = if let Some(id) = id {
        let email = EmailEntity::find_by_id(id)
            .one(connection.as_ref())
            .await?
            .ok_or(WebsiteError::NotFound)?;
        if email.account != user {
            return Err(WebsiteError::Unauthorized);
        }
        let mut email_model = email.into_active_model();
        email_model.email_address = ActiveValue::Set(email_address);
        email_model.email_type = ActiveValue::Set(email_type);
        email_model
    } else {
        if let Some(value) = emails::database_helper::get_by_address(
            connection.as_ref(),
            email_address.clone(),
            user,
        )
        .await?
        {
            if value.email_type == email_type {
                return Ok(HttpResponse::Conflict().json(value));
            } else {
                let mut email_model = value.into_active_model();
                email_model.email_address = ActiveValue::Set(email_address);
                email_model.email_type = ActiveValue::Set(email_type);
                email_model
            }
        } else {
            EmailActiveModel {
                id: Default::default(),
                account: ActiveValue::Set(user),
                email_address: ActiveValue::Set(email_address),
                email_type: ActiveValue::Set(email_type),
                created: entities::now(),
            }
        }
    };
    if email_type == EmailType::Primary {
        if let Some(value) =
            emails::database_helper::get_primary_address(connection.as_ref(), user).await?
        {
            if value.id != *email.id.as_ref() {
                debug!("If you put a primary email on an account that already has a primary email, the old primary email will be converted to an alias");
                let mut value = value.into_active_model();
                value.email_type = ActiveValue::Set(EmailType::Alias);
                value.save(connection.as_ref()).await?;
            }
        }
    }
    debug!("Saving email: {:?}", email);
    let active = email.save(connection.as_ref()).await?.try_into_model()?;
    Ok(HttpResponse::Ok().json(active))
}
#[derive(Debug, Deserialize)]
pub struct EmailAddressRemove {
    #[serde(default)]
    pub purge_emails_to_address_in_account: bool,
}
#[delete("/{user}/{email_id}")]
pub async fn delete_email(
    connection: DatabaseConnection,
    account_id: web::Path<(i64, i64)>,
    _email_address: web::Query<EmailAddressRemove>,
    auth: Authentication,
) -> Result<HttpResponse> {
    use entities::emails::Column as EmailColumn;
    if !auth.can_manage_users() {
        return Err(WebsiteError::Unauthorized);
    }

    let (user, email_id) = account_id.into_inner();
    let result: DeleteResult = EmailEntity::delete_many()
        .filter(
            EmailColumn::Id
                .eq(email_id)
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
