/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use spyland_core::{Session, State};
use sqlx::{
    SqlitePool, query,
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
};

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(options: SqliteConnectOptions) -> Result<Self> {
        Ok(Self {
            pool: SqlitePool::connect_with(options).await?,
        })
    }

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
}

pub struct SessionSql {
    pub start: i64,
    pub end: i64,

    pub is_active: bool,

    pub app_id: Option<String>,
    pub workspace: Option<i32>,
}

impl From<Session> for SessionSql {
    fn from(session: Session) -> Self {
        let is_active: bool;
        let app_id: Option<String>;
        let workspace: Option<i32>;

        if let State::Active {
            app_id: a,
            workspace: w,
        } = session.state
        {
            is_active = true;
            app_id = Some(a);
            workspace = w;
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
                    workspace: value.workspace,
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
        assert_eq!(session_sql.workspace, Some(WORKSPACE));

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
}
