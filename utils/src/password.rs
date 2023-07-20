use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

pub fn encrypt_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_ref(), &salt)
        .ok()
        .map(|v| v.to_string())
}
#[inline(always)]
pub fn check_password(password: &str, hash: &str) -> Result<bool, argon2::Error> {
    let argon2 = Argon2::default();
    let hash = match argon2::PasswordHash::new(hash) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error parsing password hash: {}", err);
            return Ok(false);
        }
    };
    if let Err(error) = argon2.verify_password(password.as_ref(), &hash) {
        match error {
            Error::Password => {}
            error => {
                log::error!("Error verifying password: {}", error);
            }
        }
        Ok(false)
    } else {
        Ok(false)
    }
}
