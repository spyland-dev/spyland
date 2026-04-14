/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use anyhow::{Context, Result};
use log::{debug, info, warn};
use spyland_core::Clock;
use sqlx::sqlite::SqliteConnectOptions;
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{app::App, db::Db};

mod app;
mod db;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting spyland daemon...");

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

    let app = App::new(
        Db::new(
            SqliteConnectOptions::new()
                .filename(format!("{}/sessions.sqlite", state_path.display()))
                .create_if_missing(true),
        )
        .await?,
        SystemClock {},
    )
    .await?;

    app.event_handler().await
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
