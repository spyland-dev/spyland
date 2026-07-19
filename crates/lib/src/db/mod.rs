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

/// Specifies the sorting order of query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort in ascending order (smallest/oldest first).
    Ascending,
    /// Sort in descending order (largest/newest first).
    Descending,
}

/// A filter structure to customize database queries for sessions.
///
/// Contains options to filter by time range, limit the number of returned entries,
/// paginate results using an offset, and specify the sorting order.
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    /// Lower bound of the time range (inclusive), represented as a Unix timestamp.
    pub from: Option<i64>,
    /// Upper bound of the time range (inclusive), represented as a Unix timestamp.
    pub to: Option<i64>,

    /// Maximum number of sessions to return.
    pub limit: Option<i64>,
    /// Number of sessions to skip before returning results (used for pagination).
    pub offset: Option<i64>,

    /// Optional sorting order based on session duration (end - start).
    pub sort_by_duration: Option<SortOrder>,
    /// Optional sorting order based on session start time.
    pub sort_by_start: Option<SortOrder>,
}

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
    ///
    /// # Arguments
    /// * `from` --- lower bound of the start time (inclusive)
    /// * `to` --- upper bound of the end time (inclusive)
    pub async fn query_range(&self, from: i64, to: i64) -> Result<Vec<SessionSql>> {
        self.query_filtered(QueryFilter {
            from: Some(from),
            to: Some(to),
            ..Default::default()
        })
        .await
    }

    /// Returns the most recent sessions, sorted descending by start time.
    ///
    /// # Arguments
    /// * `n` --- the maximum number of sessions to return
    pub async fn query_last(&self, n: i64) -> Result<Vec<SessionSql>> {
        self.query_filtered(QueryFilter {
            limit: Some(n),
            sort_by_start: Some(SortOrder::Descending),
            ..Default::default()
        })
        .await
    }

    /// Returns filtered elements from the table based on the provided filter options.
    ///
    /// # Arguments
    /// * `filter` --- the query filters, sorting, and pagination configuration to apply
    pub async fn query_filtered(&self, filter: QueryFilter) -> Result<Vec<SessionSql>> {
        let sort_duration = match filter.sort_by_duration {
            Some(SortOrder::Ascending) => Some("duration_asc"),
            Some(SortOrder::Descending) => Some("duration_desc"),
            None => None,
        };
        let sort_start = match filter.sort_by_start {
            Some(SortOrder::Ascending) => Some("start_asc"),
            Some(SortOrder::Descending) => Some("start_desc"),
            None => None,
        };

        Ok(query_as!(
            SessionSql,
            r#"
            SELECT start, end, is_active, app_id, workspace
            FROM sessions
            WHERE (?1 IS NULL OR start >= ?1)
              AND (?2 IS NULL OR end <= ?2)
            ORDER BY
              CASE WHEN ?3 = 'duration_asc' THEN (end - start) END ASC,
              CASE WHEN ?3 = 'duration_desc' THEN (end - start) END DESC,
              CASE WHEN ?4 = 'start_asc' THEN start END ASC,
              CASE WHEN ?4 = 'start_desc' THEN start END DESC,
              CASE WHEN ?3 IS NULL AND ?4 IS NULL THEN start END ASC
            LIMIT COALESCE(?5, -1) OFFSET COALESCE(?6, 0)
            "#,
            filter.from,
            filter.to,
            sort_duration,
            sort_start,
            filter.limit,
            filter.offset
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
