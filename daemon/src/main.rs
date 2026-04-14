/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use log::{debug, info, warn};

use spyland_backend_niri::NiriBackend;
use spyland_core::{Backend, Clock, SessionManager};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting spyland daemon...");

    let mut backend = new_backend().context("No backend is available")?;
    let receiver = backend.subscribe();
    let system_clock = SystemClock {};
    let mut session_manager = SessionManager::new(system_clock);

    let state_path = match env::var("XDG_STATE_HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(err) => {
            warn!("XDG_STATE_HOME is not set: {err}");
            let home = env::var("HOME").context("Home directory is not set")?;
            PathBuf::from(home).join(".local/state/")
        }
    }
    .join("spyland");

    debug!("State path: {state_path:?}");

    if !state_path.exists() {
        warn!("State path does not exist");
        std::fs::create_dir_all(&state_path).context("Failed to create state dir")?;
    }

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(format!("{}/sessions.sqlite", state_path.display()))
            .create_if_missing(true),
    )
    .await
    .context("Failed to connect to database")?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            start INTEGER NOT NULL,
            end INTEGER NOT NULL,

            is_active BOOLEAN NOT NULL,

            app_id TEXT,
            workspace INTEGER
        )",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to create database")?;

    for event in receiver {
        println!("{:?}", event);
        session_manager.handle_event(event);
    }

    Ok(())
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs()
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
