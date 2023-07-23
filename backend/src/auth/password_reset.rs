use crate::email_service::{Email, EmailAccess, EmailDebug};
use ahash::HashSet;
use chrono::{DateTime, Local};
use entities::account::panel_user::PanelUser;
use parking_lot::Mutex;
use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct PasswordResetEmail<'a> {
    pub token: &'a str,
    pub panel_url: &'a str,
    pub username: String,
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
    pub fn request(&self, user: PanelUser, panel_url: &str) {
        let token = self.generate_token();
        self.email_access.send_one_fn(
            user.backup_email.unwrap(),
            PasswordResetEmail {
                token: &token,
                panel_url: &panel_url,
                username: user.username,
            },
        );

        let mut guard = self.requests.lock();
        let request = PasswordResetRequest {
            account_id: user.id,
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
