pub mod database_helper;
pub mod full_user;
pub mod panel_user;

use sea_orm::entity::prelude::*;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use typeshare::typeshare;
use utils::database::{EmailAddress, Password};

#[derive(
    DeriveActiveEnum,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Default,
    Deserialize,
    Serialize,
    EnumString,
    Display,
    EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[typeshare]
pub enum AccountType {
    #[default]
    #[sea_orm(string_value = "individual")]
    #[strum(serialize = "individual", serialize = "personal")]
    Individual,
    #[sea_orm(string_value = "group")]
    Group,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "accounts")]
#[typeshare]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub name: String,
    #[sea_orm(unique, column_type = "Text")]
    pub username: String,
    #[sea_orm(default = "", column_type = "Text")]
    pub description: String,
    pub group_id: i64,
    #[serde(skip_serializing)]
    pub password: Password,
    #[sea_orm(default = "false")]
    pub require_password_change: bool,
    #[sea_orm(default = "0")]
    pub quota: i64,
    #[sea_orm(default_value = "Individual", column_type = "Text")]
    pub account_type: AccountType,
    #[sea_orm(default = "true")]
    pub active: bool,
    #[sea_orm(unique, nullable, column_type = "Text")]
    pub backup_email: Option<EmailAddress>,
    pub created: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}

// Foreign Key group_id to Group::id

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::groups::Entity",
        from = "Column::GroupId",
        to = "crate::groups::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Group,
    #[sea_orm(has_many = "crate::emails::Entity")]
    Email,
}
impl Related<crate::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}
impl Related<crate::emails::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Email.def()
    }
}
