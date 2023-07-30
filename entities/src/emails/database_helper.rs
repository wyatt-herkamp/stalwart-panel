use crate::emails::{Column as EmailColumn, EmailType};
use crate::{EmailActiveModel, EmailEntity, EmailModel};
use sea_orm::prelude::*;
use utils::database::EmailAddress;
pub async fn get_by_address(
    connection: &impl ConnectionTrait,
    email_address: EmailAddress,
    user_id: i64,
) -> Result<Option<EmailModel>, DbErr> {
    EmailEntity::find()
        .filter(
            EmailColumn::EmailAddress
                .eq(email_address)
                .and(EmailColumn::Account.eq(user_id)),
        )
        .one(connection)
        .await
}
pub async fn get_primary_address(
    connection: &impl ConnectionTrait,
    user_id: i64,
) -> Result<Option<EmailModel>, DbErr> {
    EmailEntity::find()
        .filter(
            EmailColumn::EmailType
                .eq(EmailType::Primary)
                .and(EmailColumn::Account.eq(user_id)),
        )
        .one(connection)
        .await
}
