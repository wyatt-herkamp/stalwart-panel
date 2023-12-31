use sea_orm::{prelude::*, sea_query::SimpleExpr, FromQueryResult, JoinType, QuerySelect};
use serde::Serialize;
use typeshare::typeshare;
use utils::database::EmailAddress;

use super::{Column as AccountColumn, Entity as AccountEntity, Relation as AccountRelation};
use crate::{
    account::AccountType,
    emails::Emails,
    groups::{Column as GroupColumn, GroupPermissions},
};
#[typeshare]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, FromQueryResult)]
pub struct FullUser {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub description: String,
    pub require_password_change: bool,
    pub quota: i64,
    pub account_type: AccountType,
    pub active: bool,
    pub backup_email: Option<EmailAddress>,
    pub created: DateTimeWithTimeZone,
    // Group Details
    pub group_id: i64,
    pub group_name: String,
    pub group_permissions: GroupPermissions,
    #[sea_orm(skip)]
    #[serde(skip_serializing_if = "Emails::is_empty")]
    pub emails: Emails,
}
impl FullUser {
    async fn get_user(
        connection: &impl ConnectionTrait,
        filter: SimpleExpr,
    ) -> Result<Option<Self>, DbErr> {
        AccountEntity::find()
            .column_as(GroupColumn::Id, "group_id")
            .column_as(GroupColumn::GroupName, "group_name")
            .column_as(GroupColumn::Permissions, "group_permissions")
            .join(JoinType::InnerJoin, AccountRelation::Group.def())
            .filter(filter)
            .into_model::<Self>()
            .one(connection)
            .await
    }
    async fn get_user_and_emails(
        connection: &impl ConnectionTrait,
        filter: SimpleExpr,
    ) -> Result<Option<Self>, DbErr> {
        let raw = Self::get_user(connection, filter).await?;
        if let Some(mut raw) = raw {
            raw.emails = Emails::get_by_user_id(connection, raw.id).await?;
            Ok(Some(raw))
        } else {
            Ok(None)
        }
    }
    pub async fn get_by_id(
        connection: &impl ConnectionTrait,
        id: i64,
        get_all_emails: bool,
    ) -> Result<Option<Self>, DbErr> {
        if get_all_emails {
            Self::get_user_and_emails(connection, AccountColumn::Id.eq(id)).await
        } else {
            Self::get_user(connection, AccountColumn::Id.eq(id)).await
        }
    }
}
