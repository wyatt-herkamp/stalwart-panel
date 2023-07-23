use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
use utils::database::EmailAddress;

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
pub enum EmailType {
    #[default]
    #[sea_orm(string_value = "primary")]
    #[strum(serialize = "primary")]
    Primary,
    #[sea_orm(string_value = "alias")]
    #[strum(serialize = "alias")]
    Alias,
    #[sea_orm(string_value = "list")]
    #[strum(serialize = "list")]
    List,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "emails")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub account: i64,
    pub email_address: EmailAddress,
    #[sea_orm(column_type = "Text")]
    pub email_type: EmailType,
    pub created: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}

// Foreign Key account to account::id

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::account::Entity",
        from = "Column::Account",
        to = "crate::account::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Account,
}

impl Related<crate::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}
