use super::{Column as AccountColumn, Entity as AccountEntity, Relation as AccountRelation};
use crate::emails::{Column as EmailColumn, EmailType};
use crate::groups::{Column as GroupColumn, GroupPermissions};

use sea_orm::prelude::*;
use sea_orm::sea_query::SimpleExpr;
use sea_orm::{FromQueryResult, JoinType, QuerySelect};
use serde::{Deserialize, Serialize};

#[derive(FromQueryResult, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PanelUser {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub active: bool,
    pub backup_email: Option<String>,
    // Group Details
    pub group_id: i32,
    pub group_name: String,
    pub group_permissions: GroupPermissions,
    // Primary Email
    pub primary_email: Option<String>,
}

impl PanelUser {
    #[inline]
    async fn get_inner(
        connection: &impl ConnectionTrait,
        filter: SimpleExpr,
    ) -> Result<Option<Self>, DbErr> {
        AccountEntity::find()
            .column_as(EmailColumn::EmailAddress, "primary_email")
            .column_as(GroupColumn::Id, "group_id")
            .column_as(GroupColumn::GroupName, "group_name")
            .column_as(GroupColumn::Permissions, "group_permissions")
            .join(JoinType::LeftJoin, crate::account::Relation::Email.def())
            .join(JoinType::InnerJoin, AccountRelation::Group.def())
            .filter(
                EmailColumn::EmailType
                    .eq(EmailType::Primary)
                    .and(AccountColumn::Active.eq(true))
                    .and(filter),
            )
            .into_model::<Self>()
            .one(connection)
            .await
    }
    pub async fn get(
        connection: &impl ConnectionTrait,
        username: &str,
    ) -> Result<Option<Self>, DbErr> {
        Self::get_inner(connection, AccountColumn::Username.eq(username)).await
    }
    pub async fn get_by_backup_email(
        connection: &impl ConnectionTrait,
        backup_email: String,
    ) -> Result<Option<Self>, DbErr> {
        Self::get_inner(connection, AccountColumn::BackupEmail.eq(backup_email)).await
    }
    pub async fn get_by_id(
        connection: &impl ConnectionTrait,
        id: i64,
    ) -> Result<Option<Self>, DbErr> {
        Self::get_inner(connection, AccountColumn::Id.eq(id)).await
    }
}
