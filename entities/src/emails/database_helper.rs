use sea_orm::prelude::*;
use utils::database::EmailAddress;

use crate::{
    emails::{Column as EmailColumn, EmailType},
    EmailEntity, EmailModel,
};
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
pub async fn does_primary_email_exist(
    connection: &impl ConnectionTrait,
    email_address: EmailAddress,
) -> Result<bool, DbErr> {
    EmailEntity::find()
        .filter(
            EmailColumn::EmailType
                .eq(EmailType::Primary)
                .and(EmailColumn::EmailAddress.eq(email_address)),
        )
        .count(connection)
        .await
        .map(|count| count > 0)
}
