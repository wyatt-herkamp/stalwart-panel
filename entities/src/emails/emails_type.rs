use std::ops::Deref;

use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, Order, QueryFilter, QueryOrder, Values,
};
use serde::Serialize;
use typeshare::typeshare;
use utils::database::EmailAddress;

use super::Column as EmailColumn;
use crate::{emails::EmailType, EmailEntity, EmailModel};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Default)]
#[typeshare]
pub struct Emails(Vec<EmailModel>);
impl Emails {
    pub async fn get_by_user_id(
        connection: &impl ConnectionTrait,
        user_id: i64,
    ) -> Result<Self, DbErr> {
        EmailEntity::find()
            .filter(EmailColumn::Account.eq(user_id))
            .order_by(
                EmailColumn::EmailType,
                Order::Field(Values(vec![EmailType::Primary.into()])),
            )
            .all(connection)
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
