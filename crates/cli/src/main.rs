/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use spyland_lib::config::{ConfigFile, ConfigSection};

#[derive(Parser)]
#[command(
    version,
    about = "Screen time for Wayland",
    long_about = "Multi-supported screen time tracking for Wayland compositors
Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
Licensed under the GNU General Public License v3.0
See source code on GitHub: https://github.com/spyland-dev/spyland",
    arg_required_else_help = true
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Config {
    sort_ascending: bool,
    sort_by_time: bool,
}

impl ConfigSection for Config {
    const SECTION: &'static str = "frontend.cli";
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sort_ascending: true,
            sort_by_time: true,
        }
    }
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Shows all your sessions in a row
    Sessions,
    /// Shows your total screen time
    Time {
        /// Sort ascending
        #[arg(short = 'A', long)]
        ascending: Option<bool>,
        /// Sort by time
        #[arg(short = 'T', long)]
        by_time: Option<bool>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    use spyland_lib::path;

    let args = Args::parse();

    let config_file = ConfigFile::new(path::ensure_config_path()?)?;

    let config: Config = config_file.get_section()?;

    match args.command {
        Command::Sessions => sessions().await,
        Command::Time { ascending, by_time } => {
            time(
                ascending.unwrap_or(config.sort_ascending),
                by_time.unwrap_or(config.sort_by_time),
            )
            .await
        }
    }
}

fn human_duration(seconds: u64) -> String {
    use std::fmt::Write;

    if seconds == 0 {
        return "0s".to_owned();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    let mut str = String::new();

    if hours > 0 {
        let _ = write!(str, "{hours}h");
    }

    if minutes > 0 {
        let _ = write!(str, "{minutes}m");
    }

    if seconds > 0 {
        let _ = write!(str, "{seconds}s");
    }

    str
}

async fn sessions() -> Result<()> {
    use spyland_core::{Session, State};
    use spyland_lib::db::Db;
    use spyland_lib::path::get_database_path;
    use time::{OffsetDateTime, UtcOffset, format_description};

    let db = Db::open_readonly(get_database_path()?).await?;

    let sessions: Vec<Session> = db
        .query_all()
        .await?
        .into_iter()
        .map(Session::from)
        .collect();

    let mut old_start = 0;

    for session in sessions {
        println!("|\n|");

        let dt = OffsetDateTime::from_unix_timestamp(session.start as i64)?;

        {
            let odt = OffsetDateTime::from_unix_timestamp(old_start)?;

            if dt.month() != odt.month() || dt.day() != odt.day() {
                println!("#    {}", {
                    let offset = UtcOffset::current_local_offset()?;
                    let format = format_description::parse(
                        "[weekday repr:short], [day] [month repr:long] [year]",
                    )?;

                    dt.to_offset(offset).format(&format)?
                });
                println!("|\n|");
            }
        }

        print!(
            "@--- ({}) {}: ",
            {
                let offset = UtcOffset::current_local_offset()?;
                let format = format_description::parse("[hour]:[minute]")?;

                dt.to_offset(offset).format(&format)?
            },
            human_duration(session.end - session.start)
        );

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
        }

        old_start = session.start as i64;
    }

    Ok(())
}

async fn time(ascending: bool, by_time: bool) -> Result<()> {
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

    let time = analytic.time_for_each_app();
    let mut stat: Vec<(&String, &u64)> = time.iter().collect();

    stat.sort_by(|x, y| {
        let cmp = if by_time {
            x.1.cmp(y.1)
        } else {
            x.0.cmp(y.0)
        };
        if ascending { cmp } else { cmp.reverse() }
    });
    for (app_id, time) in stat {
        println!("{app_id}: {}", human_duration(*time));
    }

    println!("-----");
    println!(
        "Total screen time: {}",
        human_duration(analytic.total_screen_time())
    );
    println!("Idle time: {}", human_duration(analytic.idle_time()));

    Ok(())
}
