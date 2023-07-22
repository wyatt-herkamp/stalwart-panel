pub mod database_helper;
pub mod panel_user;

use sea_orm::entity::prelude::*;
use sea_orm::strum::EnumString;

use serde::{Deserialize, Serialize};

#[derive(
    EnumIter,
    DeriveActiveEnum,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    Default,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum AccountType {
    #[default]
    #[sea_orm(string_value = "individual")]
    #[strum(serialize = "individual", serialize = "personal")]
    Individual,
    #[sea_orm(string_value = "group")]
    Group,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(default = "String::new()")]
    pub description: String,
    pub group_id: i64,
    #[serde(skip_serializing)]
    pub password: String,
    #[sea_orm(default = "0")]
    pub quota: i64,
    #[sea_orm(default)]
    pub account_type: AccountType,
    #[sea_orm(default = "true")]
    pub active: bool,
    #[sea_orm(unique)]
    pub backup_email: Option<String>,
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
