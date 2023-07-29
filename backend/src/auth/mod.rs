//pub mod middleware;

pub mod middleware;
pub mod password_reset;
pub mod permissions;
pub mod session;

use crate::auth::permissions::Permissions;
use crate::auth::session::Session;
use crate::{DatabaseConnection, Error};
use actix_web::dev::Payload;

use actix_web::{FromRequest, HttpMessage, HttpRequest};
use entities::account::panel_user::PanelUser;
use futures_util::future::LocalBoxFuture;

/// The raw authentication data.
/// Pulled from the middleware.
/// Will be converted to an [Authentication] type.
#[derive(Debug, Clone)]
pub enum AuthenticationRaw {
    Session(Session),
}
/// The authorized user.
/// Containing the user model and any additional data to the authentication method.
#[derive(Debug, Clone)]
pub enum Authentication {
    Session { user: PanelUser, session: Session },
}
impl Into<PanelUser> for Authentication {
    fn into(self) -> PanelUser {
        match self {
            Authentication::Session { user, .. } => user,
        }
    }
}

impl Authentication {}
impl Permissions for Authentication {
    fn can_manage_users(&self) -> bool {
        match self {
            Authentication::Session { user, .. } => user.group_permissions.modify_accounts,
        }
    }

    fn can_manage_system(&self) -> bool {
        match self {
            Authentication::Session { user, .. } => user.group_permissions.manage_system,
        }
    }
}
impl FromRequest for Authentication {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<AuthenticationRaw>().cloned();
        if let Some(model) = model {
            let database = req.app_data::<DatabaseConnection>().unwrap().clone();
            return Box::pin(async move {
                match model {
                    AuthenticationRaw::Session(session) => {
                        let user = PanelUser::get_by_id(database.as_ref(), session.user_id).await?;
                        if let Some(user) = user {
                            Ok(Authentication::Session { user, session })
                        } else {
                            Err(Error::Unauthorized)
                        }
                    }
                }
            });
        }
        Box::pin(async move { Err(Error::Unauthorized) })
    }
}
