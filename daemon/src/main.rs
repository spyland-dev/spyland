/*
 *  spylandd — background daemon for continuous screen time tracking
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::{Context, Result};
use log::{debug, info, warn};
use spyland_core::Clock;
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::app::App;
use spyland_lib::db::Db;

mod app;

#[tokio::main(flavor = "local")]
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

    let filename = if cfg!(debug_assertions) {
        warn!("Running in DEBUG version! Using separate database file.");
        "sessions-debug.sqlite"
    } else {
        "sessions.sqlite"
    };

    let app = App::new(
        Db::open(format!("{}/{filename}", state_path.display()), true).await?,
        SystemClock {},
    )
    .await?;

    app.run().await
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
