/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
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
