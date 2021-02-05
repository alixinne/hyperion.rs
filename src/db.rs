use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use thiserror::Error;

pub mod models;
pub mod schema;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("error conneting to the settings database: {0}")]
    Connection(#[from] diesel::ConnectionError),
    #[error("failed to find default path")]
    InvalidDefaultPath,
}

pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    pub fn try_default() -> Result<Self, DbError> {
        Ok(Self::connect(
            std::env::var("DATABASE_URL")
                .map(|v| v.to_string())
                .or_else(|_| {
                    dirs::home_dir()
                        .and_then(|path| {
                            path.join(".config/hyperion.rs/hyperion.db")
                                .to_str()
                                .map(str::to_owned)
                        })
                        .ok_or_else(|| DbError::InvalidDefaultPath)
                })?
                .as_str(),
        )?)
    }

    pub fn connect(path: &str) -> Result<Self, DbError> {
        Ok(Self {
            connection: SqliteConnection::establish(path)?,
        })
    }
}

impl std::ops::Deref for Db {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl std::ops::DerefMut for Db {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}
