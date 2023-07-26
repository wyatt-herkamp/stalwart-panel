use crate::emails::EmailType;
use crate::{EmailEntity, EmailModel};
use sea_orm::{ColumnTrait, DbBackend, Order, QueryOrder, Statement};
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use serde::Serialize;
use std::ops::Deref;
use utils::database::EmailAddress;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Default)]
pub struct Emails(Vec<EmailModel>);
impl Emails {
    pub async fn get_by_user_id(
        connection: &impl ConnectionTrait,
        user_id: i64,
    ) -> Result<Self, DbErr> {
        match connection.get_database_backend() {
            DbBackend::MySql => {
                EmailEntity::find()
                    .from_raw_sql(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        r#"SELECT * FROM "emails" WHERE "account" = ?
                                    ORDER BY case when email.email_type = 'primary' then 1 else 2 end,
                                          email.email_type ASC"#,
                        [user_id.into()],
                    ))
            }
            DbBackend::Postgres => {
                EmailEntity::find()
                    .from_raw_sql(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        r#"SELECT * FROM "emails" WHERE "account" = $1
                                    ORDER BY case when email.email_type = 'primary' then 1 else 2 end,
                                          email.email_type ASC"#,
                        [user_id.into()],
                    ))
            }
            v => unimplemented!("Unsupported database backend: {:?}", v),
        }.all(connection)
            .await
            .map(Self)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get_primary(&self) -> Option<&EmailModel> {
        self.0.iter().find(|e| e.email_type == EmailType::Primary)
    }
    pub fn get_aliases(&self) -> Vec<&EmailModel> {
        self.0
            .iter()
            .filter(|e| e.email_type == EmailType::Alias)
            .collect()
    }
    pub fn get_lists(&self) -> Vec<&EmailModel> {
        self.0
            .iter()
            .filter(|e| e.email_type == EmailType::List)
            .collect()
    }
    pub fn get_by_address(&self, address: &impl PartialEq<EmailAddress>) -> Option<&EmailModel> {
        self.0.iter().find(|e| address.eq(&e.email_address))
    }

    pub fn into_inner(self) -> Vec<EmailModel> {
        self.0
    }
}

impl Deref for Emails {
    type Target = Vec<EmailModel>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<EmailModel>> for Emails {
    fn from(v: Vec<EmailModel>) -> Self {
        Self(v)
    }
}
