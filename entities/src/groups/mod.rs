mod permissions;

use sea_orm::entity::prelude::*;

pub use permissions::GroupPermissions;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique)]
    pub group_name: String,
    #[sea_orm(column_type = "Json")]
    pub permissions: GroupPermissions,
    pub created: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}

// Foreign Key group_id to Group::id

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::account::Entity")]
    Account,
}

impl Related<crate::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}
