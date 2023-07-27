use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use redb::{CommitError, Database, Error, ReadableTable, TableDefinition};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Infallible")]
    Infallible,
    #[error("Session not found")]
    RedbError(#[from] redb::Error),
    #[error(transparent)]
    TableError(#[from] redb::TableError),
    #[error(transparent)]
    TransactionError(#[from] redb::TransactionError),
    #[error(transparent)]
    StorageError(#[from] redb::StorageError),
    #[error(transparent)]
    CommitError(#[from] CommitError),
}

/// A session type.
/// Stored in the session manager.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Session {
    pub user_id: i64,
    pub session_id: String,
    pub expires: DateTime<Local>,
    pub created: DateTime<Local>,
}
/// A tuple of (user_id, session_id, expires, created)
pub type SessionTuple<'value> = (i64, &'value str, i64, i64);
impl Session {
    pub fn from_tuple(tuple: SessionTuple) -> Self {
        let (user_id, session_id, expires, created) = tuple;
        Session {
            user_id,
            session_id: session_id.to_string(),
            expires: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_millis(expires).unwrap_or_default(),
                Utc,
            )
            .with_timezone(&Local),
            created: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_millis(created).unwrap_or_default(),
                Utc,
            )
            .with_timezone(&Local),
        }
    }
    pub fn as_tuple_ref(&self) -> SessionTuple {
        (
            self.user_id,
            self.session_id.as_str(),
            self.expires.timestamp_millis(),
            self.created.timestamp_millis(),
        )
    }
}
const TABLE: TableDefinition<&str, SessionTuple> = TableDefinition::new("sessions");

pub struct SessionManager {
    sessions: Database,
    running: AtomicBool,
}
impl SessionManager {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let sessions = if path.exists() {
            Database::open(path)?
        } else {
            Database::create(path)?
        };

        Ok(Self {
            sessions,
            running: AtomicBool::new(false),
        })
    }

    pub async fn clean_inner(&self) -> Result<(), SessionError> {
        let sessions = self.sessions.begin_write()?;

        let mut table = sessions.open_table(TABLE)?;
        let now = Local::now();
        let mut to_remove = Vec::new();
        let iter = table.iter()?;
        for index in iter {
            if let Ok((key, value)) = index {
                let session = Session::from_tuple(value.value());
                if session.expires < now {
                    to_remove.push(key.value().to_string());
                }
            }
        }
        for key in to_remove {
            if let Err(e) = table.remove(key.as_str()) {
                println!("Failed to remove session: {:?}", e); // TODO: Log
            }
        }
        drop(table);
        sessions.commit().map_err(|x| x.into())
    }
    pub fn start_cleaner(this: Arc<Self>, how_often: Duration) {
        actix_rt::spawn(async move {
            let this = this;
            let how_often = how_often.to_std().expect("Duration is too large");
            while this.running.load(Ordering::Relaxed) {
                match this.clean_inner().await {
                    Ok(_) => actix_rt::time::sleep(how_often).await,
                    Err(err) => {
                        println!("Failed to clean sessions: {:?}", err); // TODO: Log
                        actix_rt::time::sleep(how_often / 2).await
                    }
                }
            }
        });
    }
    pub fn create_session(&self, user_id: i64, life: Duration) -> Result<Session, SessionError> {
        let sessions = self.sessions.begin_write()?;
        let mut session_table = sessions.open_table(TABLE)?;

        let session_id =
            create_session_id(|x| session_table.get(x).map(|x| x.is_some()).unwrap_or(false));
        let session = Session {
            user_id,
            session_id: session_id.clone(),
            expires: Local::now() + life,
            created: Local::now(),
        };
        session_table.insert(&*session_id, session.as_tuple_ref())?;
        drop(session_table);
        sessions.commit()?;
        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.begin_read()?;

        let session = sessions.open_table(TABLE)?;
        let session = session
            .get(session_id)?
            .map(|x| Session::from_tuple(x.value()));
        Ok(session)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.begin_write()?;
        let mut table = sessions.open_table(TABLE)?;
        let session = table
            .remove(session_id)?
            .map(|x| Session::from_tuple(x.value()));
        drop(table);
        sessions.commit()?;
        Ok(session)
    }
}

#[inline(always)]
pub fn create_session_id(exists_call_back: impl Fn(&str) -> bool) -> String {
    let mut rand = StdRng::from_entropy();
    loop {
        let session_id: String = (0..7).map(|_| rand.sample(Alphanumeric) as char).collect();
        if !exists_call_back(&session_id) {
            break session_id;
        }
    }
}
