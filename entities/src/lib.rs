pub mod account;
pub mod emails;
pub mod groups;

pub use account::{
    ActiveModel as ActiveAccountModel, Entity as AccountEntity, Model as AccountModel,
};
pub use emails::{ActiveModel as ActiveEmailModel, Entity as EmailEntity, Model as EmailModel};
pub use groups::{ActiveModel as ActiveGroupModel, Entity as GroupEntity, Model as GroupModel};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue;

/// Returns an ActiveValue with the current time.
pub fn now() -> ActiveValue<DateTimeWithTimeZone> {
    ActiveValue::set(DateTimeWithTimeZone::default())
}
