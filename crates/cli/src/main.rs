/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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

use anyhow::Result;
use spyland_core::{Session, SessionAnalytics, State};
use spyland_lib::{
    config::{ConfigFile, ConfigSection},
    db::Db,
};
use std::fmt::Write;
use time::{OffsetDateTime, UtcOffset, format_description};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config_file = ConfigFile::open_default()?;

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
    let db = Db::open_default().await?;

    let sessions: Vec<Session> = db
        .query_all()
        .await?
        .into_iter()
        .map(Session::from)
        .collect();

    let mut old_start = 0;

    let offset = UtcOffset::current_local_offset()?;
    let date_format =
        format_description::parse("[weekday repr:short], [day] [month repr:long] [year]")?;
    let time_format = format_description::parse("[hour]:[minute]")?;

    let mut old_datetime: Option<OffsetDateTime> = None;

    for session in sessions {
        println!("|\n|");

        let datetime = OffsetDateTime::from_unix_timestamp(session.start as i64)?.to_offset(offset);

        if old_datetime.is_none_or(|old| datetime.date() != old.date()) {
            println!("#    {}", datetime.format(&date_format)?);
            println!("|\n|");
        }

        print!(
            "@--- ({}) {}: ",
            datetime.format(&time_format)?,
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

        old_datetime = Some(datetime);
    }

    Ok(())
}

async fn time(ascending: bool, by_time: bool) -> Result<()> {
    let db = Db::open_default().await?;

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
        let cmp = if by_time { x.1.cmp(y.1) } else { x.0.cmp(y.0) };
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
