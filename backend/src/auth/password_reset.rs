use crate::email_service::{Email, EmailAccess, EmailDebug};
use ahash::HashSet;
use chrono::{DateTime, Local};
use parking_lot::Mutex;
use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utils::database::EmailAddress;

#[derive(Debug, Serialize)]
pub struct PasswordResetEmail<'a> {
    pub token: &'a str,
    pub panel_url: &'a str,
    pub username: String,
    pub required: bool,
}
impl Email for PasswordResetEmail<'_> {
    fn template() -> &'static str {
        "password_reset"
    }

    fn subject() -> &'static str {
        "Password Reset"
    }

    fn debug_info(self) -> EmailDebug {
        EmailDebug {
            to: self.username,
            subject: Self::subject(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct PasswordResetRequest {
    pub account_id: i64,
    pub token: String,
    pub created: DateTime<Local>,
}

#[derive(Debug)]
pub struct PasswordResetManager {
    pub email_access: Arc<EmailAccess>,
    pub requests: Mutex<HashSet<PasswordResetRequest>>,
}

impl PasswordResetManager {
    pub fn request(
        &self,
        username: String,
        id: i64,
        email: EmailAddress,
        panel_url: impl AsRef<str>,
        required: bool,
    ) {
        let token = self.generate_token();
        self.email_access.send_one_fn(
            email,
            PasswordResetEmail {
                token: &token,
                panel_url: panel_url.as_ref(),
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
        guard.insert(request.clone());
    }

    pub fn get_request(&self, token: impl AsRef<str>) -> Option<PasswordResetRequest> {
        let guard = self.requests.lock();
        guard.iter().find(|r| r.token == token.as_ref()).cloned()
    }
    pub fn remove_request(&self, request: &PasswordResetRequest) {
        let mut guard = self.requests.lock();
        guard.remove(request);
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
