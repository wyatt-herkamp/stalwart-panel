use crate::email_service::{Email, EmailAccess, EmailDebug};
use ahash::{HashMap, HashSet};
use chrono::{DateTime, Local};
use parking_lot::lock_api::MappedMutexGuard;
use parking_lot::{Mutex, MutexGuard, RawMutex};
use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::sync::Arc;
use tracing::debug;
use utils::database::EmailAddress;
use crate::headers::Origin;

#[derive(Debug, Serialize)]
pub struct PasswordResetEmail<'a> {
    pub token: &'a str,
    pub panel_origin: Origin,
    pub username: String,
    pub required: bool,
}

impl Email for PasswordResetEmail<'_> {
    fn template() -> &'static str {
        "password_reset.html"
    }

    fn subject() -> &'static str {
        "Password Reset"
    }

    fn backup(&self) -> String {
        let Self {
            token,
            panel_origin,
            username,
            required,
        } = self;
        format!("Reset your password {panel_origin}/reset-password?token={token}")
    }

    fn debug_info(self) -> EmailDebug {
        EmailDebug {
            to: self.username,
            subject: Self::subject(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PasswordResetRequest {
    pub account_id: i64,
    pub token: String,
    pub created: DateTime<Local>,
}

#[derive(Debug)]
pub struct PasswordResetManager {
    pub email_access: Arc<EmailAccess>,
    pub requests: Mutex<HashMap<String, PasswordResetRequest>>,
}

impl PasswordResetManager {
    pub fn request(
        &self,
        username: String,
        id: i64,
        email: EmailAddress,
        panel_origin: Origin,
        required: bool,
    ) {
        let token = self.generate_token();
        self.email_access.send_one_fn(
            email,
            PasswordResetEmail {
                token: &token,
                panel_origin,
                username,
                required,
            },
        );

        let mut guard = self.requests.lock();
        let request = PasswordResetRequest {
            account_id: id,
            token,
            created: Local::now(),
        };
        guard.insert(request.token.clone(), request);

        debug!("{:?}", guard);
    }

    pub fn get_request(
        &self,
        token: impl AsRef<str>,
    ) -> Option<impl DerefMut<Target = PasswordResetRequest> + '_> {
        MutexGuard::try_map(self.requests.lock(), |mut r| r.get_mut(token.as_ref())).ok()
    }
    pub fn remove_request(&self, request: impl AsRef<str>) {
        let mut guard = self.requests.lock();
        guard.remove(request.as_ref());
    }

    fn generate_token(&self) -> String {
        let mut rng = rand::rngs::OsRng::default();
        rand::distributions::Alphanumeric
            .sample_iter(&mut rng)
            .take(36)
            .map(char::from)
            .collect()
    }
}
