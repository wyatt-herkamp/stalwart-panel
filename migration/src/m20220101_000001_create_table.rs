use sea_orm_migration::prelude::*;

use crate::sea_orm::Schema;

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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(TableDropStatement::new().table(Emails::Table).to_owned())
            .await?;
        manager
            .drop_table(TableDropStatement::new().table(Accounts::Table).to_owned())
            .await?;
        manager
            .drop_table(TableDropStatement::new().table(Groups::Table).to_owned())
            .await?;
        Ok(())
    }
}
#[derive(Iden)]
pub enum Groups {
    Table,
}

#[derive(Iden)]
pub enum Accounts {
    Table,
}

#[derive(Iden)]
pub enum Emails {
    Table,
}
