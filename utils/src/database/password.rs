use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

use argon2::{
    password_hash::{Error, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rand::{distributions::Distribution, prelude::StdRng, rngs::OsRng, SeedableRng};
use serde::{Deserialize, Serialize};
use strum::{Display as StrumDisplay, IntoStaticStr};
use thiserror::Error;

/// Hash Types supported by Stalwart [More Info](https://stalw.art/docs/directory/users#passwords)
///
/// Types [PasswordType::PlainText] and [PasswordType::None] are not checked by panel
/// and will always return false
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    IntoStaticStr,
    StrumDisplay,
    Serialize,
    Deserialize,
    Default,
)]
pub enum PasswordType {
    /// Argon2 Hash. Recommended and What Stalwart Panel uses
    #[default]
    Argon2,
    /// Please refer to [PasswordType::SHA512]
    SHA256,
    /// This algorithm can burn in hell for all I care
    ///
    /// I lost an hour of my life because it had a false negative
    SHA512,
    /// I am not letting this password be checked. It is too insecure.
    /// This element exists just for the importing of old passwords
    ///
    /// I like my code to be secure unlike me as a person
    PlainText,
    /// Default for all other types. This is so upon importing it won't crash
    None,
}
#[derive(Debug, Error)]
pub enum PasswordErrors {
    #[error("Internal password error: {0}")]
    InternalPasswordError(String),
    #[error("Unsupported hash type: {0}")]
    UnsupportedHashType(PasswordType),
}
impl From<argon2::Error> for PasswordErrors {
    fn from(value: argon2::Error) -> Self {
        PasswordErrors::InternalPasswordError(value.to_string())
    }
}
impl From<argon2::password_hash::Error> for PasswordErrors {
    fn from(value: argon2::password_hash::Error) -> Self {
        PasswordErrors::InternalPasswordError(value.to_string())
    }
}
impl PasswordType {
    pub fn identify(password: &str) -> Self {
        if password.starts_with("$argon2") {
            PasswordType::Argon2
        } else if password.starts_with("$5$") {
            PasswordType::SHA256
        } else if password.starts_with("$6$") {
            PasswordType::SHA512
        } else if password.starts_with("{PLAIN}")
            || password.starts_with("{plain}")
            || password.starts_with("{clear}")
            || password.starts_with("{CLEAR}")
        {
            PasswordType::PlainText
        } else {
            PasswordType::None
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Password {
    pub(crate) password: String,
    pub(crate) hash_type: PasswordType,
}
impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Password")
            .field("hash_type", &self.hash_type)
            .finish()
    }
}
impl Password {
    /// Creates a new Password from the AlphaNumeric characters of length 16
    /// This is used for creating a new password
    ///
    /// Returns the password and the string
    pub fn create_password(method: PasswordType) -> Result<(Self, String), PasswordErrors> {
        let mut rng = StdRng::from_entropy();
        let password_str: String = rand::distributions::Alphanumeric
            .sample_iter(&mut rng)
            .take(16)
            .map(char::from)
            .collect();
        let password = Self::new_hash(&password_str, method)?;
        Ok((password, password_str))
    }
    /// Hashes the password using the specified method
    /// Should only be used on PasswordType::PlainText
    pub fn hash(&mut self, method: PasswordType) -> Result<(), PasswordErrors> {
        //Get all characters after the first 6
        let password = &self.password[6..];
        let result = Self::new_hash(password, method)?;
        *self = result;
        Ok(())
    }
    /// Takes a password and hashes it using the specified method
    pub fn new_hash(
        password: impl AsRef<str>,
        method: PasswordType,
    ) -> Result<Self, PasswordErrors> {
        match method {
            PasswordType::Argon2 => {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password = password.as_ref();
                let hash = argon2.hash_password(password.as_bytes(), &salt)?;
                Ok(Password {
                    password: hash.to_string(),
                    hash_type: PasswordType::Argon2,
                })
            }
            _ => Err(PasswordErrors::UnsupportedHashType(method)),
        }
    }
    pub fn new_argon2(password: impl AsRef<[u8]>) -> Result<Self, PasswordErrors> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_ref(), &salt)
            .map(|v| Password {
                password: v.to_string(),
                hash_type: PasswordType::Argon2,
            })
            .map_err(PasswordErrors::from)
    }
    pub fn new_hashed(password: impl Into<String>) -> Self {
        let password = password.into();
        let hash_type = PasswordType::identify(&password);
        Password {
            password,
            hash_type,
        }
    }
    pub fn check_password(&self, password: impl AsRef<str>) -> Result<bool, PasswordErrors> {
        match &self.hash_type {
            PasswordType::Argon2 => self
                .check_argon2(password.as_ref())
                .map_err(PasswordErrors::from),
            PasswordType::SHA256 => self.check_sha256_crypt(password),
            PasswordType::SHA512 => self.check_sha512_crypt(password),
            v => {
                log::error!("Unsupported hash type: {:?}", self.hash_type);
                Err(PasswordErrors::UnsupportedHashType(v.clone()))
            }
        }
    }
    fn check_sha512_crypt(&self, password: impl AsRef<str>) -> Result<bool, PasswordErrors> {
        return match sha_crypt::sha512_check(password.as_ref(), &self.password) {
            Ok(_) => Ok(true),
            Err(_e) => Ok(false),
        };
    }

    fn check_sha256_crypt(&self, password: impl AsRef<str>) -> Result<bool, PasswordErrors> {
        match sha_crypt::sha256_check(password.as_ref(), &self.password) {
            Ok(_) => Ok(true),
            Err(_e) => Ok(false),
        }
    }

    fn check_argon2(&self, password: impl AsRef<[u8]>) -> Result<bool, argon2::Error> {
        let argon2 = Argon2::default();
        let hash = match argon2::PasswordHash::new(&self.password) {
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
    pub fn hash_type(&self) -> PasswordType {
        self.hash_type
    }
}

impl FromStr for Password {
    type Err = PasswordErrors;

    fn from_str(s: &str) -> Result<Password, Self::Err> {
        Password::new_argon2(s)
    }
}

impl TryFrom<String> for Password {
    type Error = PasswordErrors;

    fn try_from(value: String) -> Result<Password, Self::Error> {
        Password::new_argon2(value)
    }
}
impl Into<String> for Password {
    fn into(self) -> String {
        self.password
    }
}

impl Deref for Password {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.password
    }
}
impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.password
    }
}
impl AsRef<String> for Password {
    fn as_ref(&self) -> &String {
        &self.password
    }
}
impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED] (Hash Type: {})", self.hash_type)
    }
}

#[cfg(feature = "sea-orm")]
mod database {
    use sea_orm::{
        sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr},
        ColIdx, ColumnType, QueryResult, TryGetError, TryGetable, Value,
    };

    use crate::database::Password;

    impl From<Password> for Value {
        fn from(value: Password) -> Self {
            Value::String(Some(value.password.into()))
        }
    }
    impl TryGetable for Password {
        fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
            let value = String::try_get_by(res, index)?;
            Ok(Password::new_hashed(value))
        }
    }
    impl ValueType for Password {
        fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
            match v {
                Value::String(Some(s)) => Ok(Password::new_hashed(*s)),
                _ => Err(ValueTypeErr),
            }
        }

        fn type_name() -> String {
            stringify!(Password).to_owned()
        }

        fn array_type() -> ArrayType {
            ArrayType::String
        }

        fn column_type() -> ColumnType {
            ColumnType::String(None)
        }
    }

    impl Nullable for Password {
        fn null() -> Value {
            Value::String(None)
        }
    }
}

/// Implements serde for Password
///
/// This will redact the password when serializing
/// On deserialization, it will hash the password
mod _serde {
    use log::warn;

    use crate::database::password::Password;

    impl serde::Serialize for Password {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            warn!("Something is trying to serialize a password! Please report this!");
            serializer.serialize_none()
        }
    }

    impl<'de> serde::Deserialize<'de> for Password {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Password::new_argon2(s).map_err(serde::de::Error::custom)
        }
    }
}
