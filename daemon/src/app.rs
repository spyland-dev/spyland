/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::sync::mpsc::Receiver;

use spyland_backend_niri::NiriBackend;
use spyland_core::{Backend, Clock, Event, SessionManager};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use anyhow::{Context, Result};

pub struct App<C: Clock> {
    receiver: Receiver<Event>,
    session_manager: SessionManager<C>,
    db_pool: SqlitePool,
}

impl<C: Clock> App<C> {
    pub async fn new(sqlite_options: SqliteConnectOptions, clock: C) -> Result<Self> {
        let mut backend = new_backend().context("No backend is available")?;

        let db_pool = SqlitePool::connect_with(sqlite_options)
            .await
            .context("Failed to connect to database")?;

        sqlx::query(
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
        .fetch_all(&db_pool)
        .await
        .context("Failed to create database")?;

        Ok(Self {
            receiver: backend.subscribe(),
            session_manager: SessionManager::new(clock),
            db_pool,
        })
    }

    pub async fn event_handler(mut self) -> Result<()> {
        for event in self.receiver {
            println!("{:?}", event);
            self.session_manager.handle_event(event);
        }

        Ok(())
    }
}

fn new_backend() -> Option<Box<dyn Backend>> {
    let backends: Vec<Box<dyn Backend>> = vec![Box::new(NiriBackend::default())];

    for backend in backends {
        if backend.is_available() {
            return Some(backend);
        }
    }

    None
}
