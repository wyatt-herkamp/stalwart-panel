use crate::sea_orm::ActiveValue::Set;
use crate::sea_orm::Schema;
use entities::groups::GroupPermissions;
use entities::GroupEntity;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{EntityTrait, NotSet};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());
        crate::entities!(schema, manager, entities::GroupEntity);
        crate::entities!(schema, manager, entities::AccountEntity);
        crate::entities!(schema, manager, entities::EmailEntity);

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
