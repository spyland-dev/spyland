/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(
    version,
    about = "Screen time for Wayland",
    long_about = "Multi-supported screen time tracking for Wayland compositors
Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
Licensed under the GNU General Public License v3.0
See source code on GitHub: https://github.com/NonExistPlayer/spyland",
    arg_required_else_help = true
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Shows all your sessions in a row
    Sessions,
    /// Shows your total screen time
    Time,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let result = match args.command {
        Command::Sessions => sessions().await,
        Command::Time => time().await,
    };

    if let Err(err) = result {
        eprintln!("{err:#}");
    }
}

async fn sessions() -> Result<()> {
    use spyland_core::{Session, State};
    use spyland_lib::db::Db;
    use spyland_lib::path::get_database_path;

    let db = Db::open_readonly(get_database_path()?).await?;

    let sessions: Vec<Session> = db
        .query_all()
        .await?
        .into_iter()
        .map(Session::from)
        .collect();

    for session in sessions {
        println!("|\n|");
        print!("@--- {}s: ", session.utc_end - session.utc_start);

        match &session.state {
            State::Active { app_id, workspace } => {
                print!("{app_id}");
                if let Some(w) = workspace {
                    print!(", {w}");
                }
                println!();
            }
            State::Idle => {
                println!("Idle");
            }
            State::Empty => unreachable!(),
        }
    }

    Ok(())
}

async fn time() -> Result<()> {
    use spyland_core::{Session, SessionAnalytics};
    use spyland_lib::db::Db;
    use spyland_lib::path::get_database_path;

    let db = Db::open_readonly(get_database_path()?).await?;

    let sessions: Vec<Session> = db
        .query_all()
        .await?
        .into_iter()
        .map(Session::from)
        .collect();

    let analytic = SessionAnalytics::new(sessions);

    println!("Total screen time: {}s", analytic.total_screen_time());
    println!("Idle time: {}s", analytic.idle_time());

    Ok(())
}
