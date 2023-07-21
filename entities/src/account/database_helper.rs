use crate::account::AccountType;
use crate::emails::EmailType;
use crate::{AccountEntity, AccountModel, EmailEntity, EmailModel};
use sea_orm::prelude::*;
use sea_orm::{FromQueryResult, JoinType, QuerySelect};
use serde::{Deserialize, Serialize};

pub type AccountWithEmails = (AccountModel, Vec<EmailModel>);
use crate::emails::Column as EmailColumn;

pub async fn get_account_with_associated_emails_by_id(
    connection: &impl ConnectionTrait,
    id: i64,
) -> Result<Option<AccountWithEmails>, DbErr> {
    let account = AccountEntity::find_by_id(id).one(connection).await?;
    if let Some(account) = account {
        let emails = EmailEntity::find()
            .filter(EmailColumn::Account.eq(id))
            .all(connection)
            .await?;
        Ok(Some((account, emails)))
    } else {
        Ok(None)
    }
}

/// Good for a list of all accounts
///
/// This contains the primary email address for the account if it exists
#[derive(FromQueryResult, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountSimple {
    pub id: i32,
    pub username: String,
    pub description: String,
    pub account_type: AccountType,
    pub primary_email: Option<String>,
}
impl AccountSimple {
    /// Get all accounts active or not
    pub async fn get_all_accounts(
        connection: &impl ConnectionTrait,
    ) -> Result<Vec<AccountSimple>, DbErr> {
        AccountEntity::find()
            .select_only()
            .columns(vec![
                crate::account::Column::Id,
                crate::account::Column::Username,
                crate::account::Column::Description,
                crate::account::Column::AccountType,
            ])
            .column_as(EmailColumn::EmailAddress, "primary_email")
            .filter(EmailColumn::EmailType.eq(EmailType::Primary))
            .join(JoinType::LeftJoin, crate::account::Relation::Email.def())
            .into_model::<AccountSimple>()
            .all(connection)
            .await
    }
    /// Get all active accounts
    pub async fn get_all_active_accounts(
        connection: &impl ConnectionTrait,
    ) -> Result<Vec<AccountSimple>, DbErr> {
        AccountEntity::find()
            .select_only()
            .columns(vec![
                crate::account::Column::Id,
                crate::account::Column::Username,
                crate::account::Column::Description,
                crate::account::Column::AccountType,
            ])
            .column_as(EmailColumn::EmailAddress, "primary_email")
            .filter(
                EmailColumn::EmailType
                    .eq(EmailType::Primary)
                    .and(crate::account::Column::Active.eq(true)),
            )
            .join(JoinType::LeftJoin, crate::account::Relation::Email.def())
            .into_model::<AccountSimple>()
            .all(connection)
            .await
    }
}
