use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::str::FromStr;
use strum_macros::{Display as StrumDisplay, IntoStaticStr};
use thiserror::Error;

/// Hash Types supported by Stalwart [Info](https://stalw.art/docs/directory/users#passwords)
///
/// TODO: Add all supported hash types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IntoStaticStr, StrumDisplay)]
pub enum PasswordHash {
    /// Argon2 Hash. Recommended and What Stalwart Panel uses
    Argon2,
    SHA256,
    SHA512,
    None,
}
impl Default for PasswordHash {
    fn default() -> Self {
        PasswordHash::Argon2
    }
}
#[derive(Debug, Error)]
pub enum PasswordErrors {
    #[error("Internal password error: {0}")]
    InternalPasswordError(String),
    #[error("Unsupported hash type: {0}")]
    UnsupportedHashType(PasswordHash),
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
impl PasswordHash {
    pub fn identify(password: &str) -> Self {
        if password.starts_with("$argon2") {
            PasswordHash::Argon2
        } else if password.starts_with("$5$") {
            PasswordHash::SHA256
        } else if password.starts_with("$6$") {
            PasswordHash::SHA512
        } else {
            PasswordHash::None
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Password {
    pub(crate) password: String,
    pub(crate) hash_type: PasswordHash,
}
impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Password")
            .field("hash_type", &self.hash_type)
            .finish()
    }
}
impl Password {
    pub fn new_argon2(password: impl AsRef<[u8]>) -> Result<Self, PasswordErrors> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_ref(), &salt)
            .map(|v| Password {
                password: v.to_string(),
                hash_type: PasswordHash::Argon2,
            })
            .map_err(PasswordErrors::from)
    }
    pub fn new_hashed(password: impl Into<String>) -> Self {
        let password = password.into();
        let hash_type = PasswordHash::identify(&password);
        Password {
            password,
            hash_type,
        }
    }
    pub fn check_password(&self, password: impl AsRef<str>) -> Result<bool, PasswordErrors> {
        match &self.hash_type {
            PasswordHash::Argon2 => self
                .check_argon2(password.as_ref())
                .map_err(PasswordErrors::from),
            PasswordHash::SHA256 => self.check_sha256_crypt(password),
            PasswordHash::SHA512 => self.check_sha512_crypt(password),
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
    use crate::database::Password;
    use sea_orm::sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr};
    use sea_orm::{ColIdx, ColumnType, QueryResult, TryGetError, TryGetable, Value};

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
    use crate::database::password::Password;
    use log::warn;

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
