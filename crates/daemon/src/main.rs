/*
 *  spylandd — background daemon for continuous screen time tracking
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use log::info;
use spyland_core::manager::Clock;
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::app::App;
use clap::Parser;
use spyland_lib::{db::Db, ipc::IpcServer, path};

mod app;

#[derive(Parser)]
#[command(
    version,
    about = "Screen time daemon for Wayland",
    long_about = "Background daemon for continuous screen time tracking
Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
Licensed under the GNU General Public License v3.0
See source code on GitHub: https://github.com/spyland-dev/spyland"
)]
struct Cli {
    /// Path to database file
    #[arg(short = 'D', long)]
    database: Option<PathBuf>,
    /// Path to socket
    #[arg(short = 'S', long)]
    socket: Option<PathBuf>,
    /// Path to config file
    #[arg(short = 'C', long)]
    config: Option<PathBuf>,

    /// Path to executable backend to start
    #[arg(short = 'B', long, env = "SPYLAND_BACKEND")]
    backend: Option<PathBuf>,

    /// Sets log level
    #[arg(short, long, env = "RUST_LOG")]
    log_level: Option<log::LevelFilter>,
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let mut builder = env_logger::Builder::from_default_env();

    if let Some(log) = args.log_level {
        builder.filter_level(log);
    }

    builder.init();

    if let Some(backend) = args.backend {
        use std::process::Command;

        info!("Starting new backend '{}'", backend.display());

        Command::new(backend).spawn()?;
    }

    info!("Starting spyland daemon...");

    let app = App::new(
        Db::open(
            match args.database {
                None => path::ensure_database_path()?,
                Some(path) => path,
            },
            true,
        )
        .await?,
        IpcServer::new(match args.socket {
            None => path::ensure_socket_path()?,
            Some(path) => path,
        })?,
        match args.config {
            None => path::ensure_config_path()?,
            Some(path) => path,
        },
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
