use std::sync::Arc;
use ahash::HashSet;
use chrono::{DateTime, Local};
use parking_lot::Mutex;
use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};
use entities::account::panel_user::PanelUser;
use crate::email_service::EmailAccess;

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
    pub panel_url: String,
}

impl PasswordResetManager{

    pub fn request(&self, user: PanelUser){
        let token = self.generate_token();
        {
            let mut guard = self.requests.lock();
            let request = PasswordResetRequest {
                account_id: user.id,
                token: token.clone(),
                created: Local::now(),
            };
            guard.insert(request.clone());
        }

        let url = format!("{}/reset/password/{}", self.panel_url, token);
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