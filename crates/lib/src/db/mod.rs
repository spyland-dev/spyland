/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Module to work with spyland database.

pub use crate::db::sessql::SessionSql;
use std::path::Path;

use anyhow::Result;
use sqlx::{
    SqlitePool, query, query_as,
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
};

mod sessql;

#[cfg(test)]
mod tests;

/// Useful wrapper to manage database.
///
/// # Example
/// ```ignore
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
    pub async fn from_options(options: SqliteConnectOptions) -> Result<Self> {
        Ok(Self {
            pool: SqlitePool::connect_with(options).await?,
        })
    }

    /// Creates [Db] from the [SqlitePool].
    pub async fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Opens database by its path.
    ///
    /// # Arguments
    /// * `path` --- path to the SQLite database file
    /// * `create_if_missing` --- creates file if its missing
    ///
    /// # Example
    /// ```ignore
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

    #[cfg(feature = "path")]
    /// Opens database by [`crate::path::ensure_database_path`].
    pub async fn open_default() -> Result<Self> {
        Self::open(crate::path::ensure_database_path()?, false).await
    }

    /// Read-only opens database by its path.
    ///
    /// # Example
    /// ```ignore
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

    /// Returns all elements from the table that intersect with the specified range.
    pub async fn query_range(&self, from: i64, to: i64) -> Result<Vec<SessionSql>> {
        Ok(query_as!(
            SessionSql,
            "SELECT * FROM sessions WHERE start < ?1 AND end > ?2",
            to,
            from
        )
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
    /// ```ignore
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
    /// ```ignore
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
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
