/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Module to work with spyland database.

use std::path::Path;

use anyhow::Result;
use spyland_core::{Session, State};
use sqlx::{
    SqlitePool, query, query_as,
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
};

/// Useful wrapper to manage database.
///
/// # Example
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// // Opens database file
/// let db = Db::open("/path/to/database.sqlite").await?;
///
/// db.create().await?; // Creating if not exists
///
/// let session: Session;
/// db.insert(session.into()).await?; // Inserting session
/// # }
/// ```
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    /// Creates [`Db`] from [`SqliteConnectOptions`].
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = Db::from_options(
    ///     SqliteConnectOptions::new()
    ///         .in_memory()
    /// ).await?;
    /// # }
    /// ```
    pub async fn from_options(options: SqliteConnectOptions) -> Result<Self> {
        Ok(Self {
            pool: SqlitePool::connect_with(options).await?,
        })
    }

    /// Opens database by its path.
    ///
    /// # Arguments
    /// * `path` --- path to the SQLite database file
    /// * `create_if_missing` --- creates file if its missing
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = Db::open("/path/to/database.sqlite").await?;
    /// # }
    /// ```
    pub async fn open(path: impl AsRef<Path>, create_if_missing: bool) -> Result<Self> {
        Ok(Self {
            pool: SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(create_if_missing),
            )
            .await?,
        })
    }

    /// Read-only opens database by its path.
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let db = Db::open_readonly("/path/to/database.sqlite")
    /// # }
    /// ```
    pub async fn open_readonly(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            pool: SqlitePool::connect_with(
                SqliteConnectOptions::new().filename(path).read_only(true),
            )
            .await?,
        })
    }

    /// Creates table if not exists.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let db = Db::from_options(
    /// #     SqliteConnectOptions::new()
    /// #         .in_memory()
    /// # ).await?;
    /// db.create().await?;
    /// # }
    /// ```
    pub async fn create(&self) -> Result<()> {
        query!(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                start INTEGER NOT NULL,
                end INTEGER NOT NULL,

                is_active BOOLEAN NOT NULL,

                app_id TEXT,
                workspace INTEGER
            )
            ",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(())
    }

    /// Inserts [`SessionSql`] to the table.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let db = Db::from_options(
    /// #     SqliteConnectOptions::new()
    /// #         .in_memory()
    /// # ).await?;
    /// db.create().await?;
    ///
    /// # let session = Session {
    /// #   utc_start: 0,
    /// #   utc_end: 15,
    /// #   state: State::Idle,
    /// # };
    ///
    /// // Don't forget `.into()`!
    /// db.insert(session.into()).await?;
    /// # }
    /// ```
    pub async fn insert(&self, session: SessionSql) -> Result<SqliteQueryResult> {
        let result = query!(
            "
            INSERT INTO sessions (start, end, is_active, app_id, workspace)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            session.start,
            session.end,
            session.is_active,
            session.app_id,
            session.workspace,
        )
        .execute(&self.pool)
        .await?;

        Ok(result)
    }

    /// Returns all elements from the table.
    pub async fn query_all(&self) -> Result<Vec<SessionSql>> {
        Ok(query_as!(SessionSql, "SELECT * FROM sessions")
            .fetch_all(&self.pool)
            .await?)
    }

    /// Updates a session entry by its row ID.
    ///
    /// # Arguments
    /// * `rowid` --- the internal SQLite row ID
    /// * `session` --- the updated session data
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let db = Db::open("/path/to/database.sqlite", true).await?;
    /// # db.create().await?;
    /// # let session = /* ... */;
    /// // Update session at row 1
    /// db.update_by_rowid(1, session).await?;
    /// # }
    /// ```
    pub async fn update_by_rowid(
        &self,
        rowid: i64,
        session: SessionSql,
    ) -> Result<SqliteQueryResult> {
        let result = query!(
            "
            UPDATE sessions
            SET start = ?1, end = ?2, is_active = ?3, app_id = ?4, workspace = ?5
            WHERE rowid = ?6
            ",
            session.start,
            session.end,
            session.is_active,
            session.app_id,
            session.workspace,
            rowid,
        )
        .execute(&self.pool)
        .await?;

        Ok(result)
    }

    /// Updates the last (most recent) session entry.
    ///
    /// This updates the session with the highest `rowid`.
    ///
    /// # Arguments
    /// * `session` --- the updated session data
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let db = Db::open("/path/to/database.sqlite", true).await?;
    /// # db.create().await?;
    /// # let session = /* ... */;
    /// // Update the most recent session
    /// db.update_last(session).await?;
    /// # }
    /// ```
    pub async fn update_last(&self, session: SessionSql) -> Result<SqliteQueryResult> {
        let result = query!(
            "
            UPDATE sessions
            SET start = ?1, end = ?2, is_active = ?3, app_id = ?4, workspace = ?5
            WHERE rowid = (SELECT MAX(rowid) FROM sessions)
            ",
            session.start,
            session.end,
            session.is_active,
            session.app_id,
            session.workspace,
        )
        .execute(&self.pool)
        .await?;

        Ok(result)
    }
}

/// A database representation for [`Session`].
///
/// Used to convert and store [`Session`] data in SQLite.
pub struct SessionSql {
    /// Start time in seconds
    pub start: i64,
    /// End time in seconds
    pub end: i64,

    /// Is active session
    ///
    /// This field determines some other fields:
    /// - [`SessionSql::app_id`] will only have a value ([`Some`]) if this field equals `true`.
    /// - [`SessionSql::workspace`] will never have a value ([`None`]) if this field equals
    /// `false`.
    /// See more documentation for these fields.
    pub is_active: bool,

    /// Application identifier.
    ///
    /// Only [`Some`] if this is an active session.
    /// See [`SessionSql::is_active`]
    pub app_id: Option<String>,
    /// Workspace number.
    ///
    /// Unlike [`SessionSql::app_id`], it may be [`None`] even if [`SessionSql::is_active`] equals
    /// `true`, because of some compositors may not have workspaces at all (see [`State`]).
    pub workspace: Option<i64>,
}

impl From<Session> for SessionSql {
    fn from(session: Session) -> Self {
        let is_active: bool;
        let app_id: Option<String>;
        let workspace: Option<i64>;

        if let State::Active {
            app_id: a,
            workspace: w,
        } = session.state
        {
            is_active = true;
            app_id = Some(a);
            workspace = match w {
                Some(i) => Some(i as i64),
                None => None,
            };
        } else {
            is_active = false;
            app_id = None;
            workspace = None;
        }

        Self {
            start: session.utc_start as i64,
            end: session.utc_end as i64,

            is_active,

            app_id,
            workspace,
        }
    }
}

impl From<SessionSql> for Session {
    fn from(value: SessionSql) -> Self {
        if value.is_active {
            Self {
                utc_start: value.start as u64,
                utc_end: value.end as u64,

                state: State::Active {
                    app_id: value.app_id.unwrap(),
                    workspace: match value.workspace {
                        Some(i) => Some(i as i32),
                        None => None,
                    },
                },
            }
        } else {
            Self {
                utc_start: value.start as u64,
                utc_end: value.end as u64,

                state: State::Idle,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use spyland_core::{Session, State};
    use sqlx::{SqlitePool, query};

    use crate::db::{Db, SessionSql};

    #[sqlx::test]
    async fn create_test(pool: SqlitePool) {
        let db = Db { pool };

        db.create().await.unwrap();
    }

    #[sqlx::test]
    async fn insert_test(pool: SqlitePool) {
        let db = Db { pool };

        db.create().await.unwrap();

        let session = Session {
            utc_start: 1,
            utc_end: 16,
            state: State::Active {
                app_id: "example_test_app_id".into(),
                workspace: None,
            },
        };

        let result = db.insert(session.into()).await.unwrap();

        assert_eq!(result.rows_affected(), 1);
    }

    #[sqlx::test]
    async fn insert_integrity_test(pool: SqlitePool) {
        let db = Db { pool };

        db.create().await.unwrap();

        const START: u64 = 1;
        const END: u64 = 31;
        const APP_ID: &str = "steam";
        const WORKSPACE: i32 = 3;

        let session = Session {
            utc_start: START,
            utc_end: END,
            state: State::Active {
                app_id: APP_ID.into(),
                workspace: Some(WORKSPACE),
            },
        };

        let result = db.insert(session.into()).await.unwrap();

        assert_eq!(result.rows_affected(), 1);

        let result = query!("SELECT * FROM sessions")
            .fetch_one(&db.pool)
            .await
            .unwrap();

        assert_eq!(result.start, START as i64);
        assert_eq!(result.end, END as i64);
        assert_eq!(result.is_active, true);
        assert_eq!(result.app_id, Some(APP_ID.into()));
        assert_eq!(result.workspace, Some(WORKSPACE as i64));
    }

    #[test]
    fn session_mapping_test() {
        const START: u64 = 1;
        const END: u64 = 16;

        const APP_ID: &str = "example_test_app_id";
        const WORKSPACE: i32 = 2;

        let session = Session {
            utc_start: START,
            utc_end: END,

            state: State::Active {
                app_id: APP_ID.into(),
                workspace: Some(WORKSPACE),
            },
        };

        let session_sql: SessionSql = session.into();

        assert_eq!(session_sql.start, START as i64);
        assert_eq!(session_sql.end, END as i64);
        assert_eq!(session_sql.is_active, true);
        assert_eq!(session_sql.app_id, Some(APP_ID.into()));
        assert_eq!(session_sql.workspace, Some(WORKSPACE as i64));

        let session2: Session = session_sql.into();

        assert_eq!(session2.utc_start, START);
        assert_eq!(session2.utc_end, END);
        assert!(matches!(
            session2.state,
            State::Active {
                app_id,
                workspace: Some(WORKSPACE),
            } if app_id == APP_ID
        ));
    }

    #[sqlx::test]
    async fn update_by_rowid_test(pool: SqlitePool) {
        let db = Db { pool };

        db.create().await.unwrap();

        let session1 = Session {
            utc_start: 1,
            utc_end: 16,
            state: State::Active {
                app_id: "firefox".into(),
                workspace: Some(1),
            },
        };

        db.insert(session1.into()).await.unwrap();

        const UPDATED_END: i64 = 50;
        const UPDATED_APP_ID: &str = "chromium";

        let updated_session = SessionSql {
            start: 1,
            end: UPDATED_END,
            is_active: true,
            app_id: Some(UPDATED_APP_ID.into()),
            workspace: Some(1),
        };

        let result = db.update_by_rowid(1, updated_session).await.unwrap();
        assert_eq!(result.rows_affected(), 1);

        let sessions = db.query_all().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].end, UPDATED_END);
        assert_eq!(sessions[0].app_id, Some("chromium".into()));
    }

    #[sqlx::test]
    async fn update_last_test(pool: SqlitePool) {
        let db = Db { pool };

        db.create().await.unwrap();

        const S1_END: u64 = 20;

        let session1 = Session {
            utc_start: 1,
            utc_end: S1_END,
            state: State::Active {
                app_id: "firefox".into(),
                workspace: None,
            },
        };

        const START: u64 = 20;
        const APP_ID: &str = "steam";
        const S2_END: u64 = 60;

        let session2 = Session {
            utc_start: START,
            utc_end: 40,
            state: State::Active {
                app_id: APP_ID.into(),
                workspace: Some(2),
            },
        };

        db.insert(session1.into()).await.unwrap();
        db.insert(session2.into()).await.unwrap();

        let updated_last = SessionSql {
            start: START as i64,
            end: S2_END as i64,
            is_active: true,
            app_id: Some(APP_ID.into()),
            workspace: Some(2),
        };

        let result = db.update_last(updated_last).await.unwrap();
        assert_eq!(result.rows_affected(), 1);

        let sessions = db.query_all().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].end, S1_END as i64);
        assert_eq!(sessions[1].end, S2_END as i64);
    }
}
