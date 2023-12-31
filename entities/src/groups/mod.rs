mod permissions;

pub use permissions::GroupPermissions;
use sea_orm::entity::prelude::*;
use serde::Serialize;
use typeshare::typeshare;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "groups")]
#[typeshare]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique, column_type = "Text")]
    pub group_name: String,
    #[sea_orm(column_type = "Json")]
    pub permissions: GroupPermissions,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
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
