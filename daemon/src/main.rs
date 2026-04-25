/*
 *  spylandd — background daemon for continuous screen time tracking
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use log::info;
use spyland_core::Clock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::App;
use spyland_lib::{db::Db, ipc::IpcServer, path};

mod app;

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting spyland daemon...");

    let app = App::new(
        Db::open(path::ensure_database_path()?, true).await?,
        IpcServer::new(path::ensure_socket_path()?.into())?,
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
