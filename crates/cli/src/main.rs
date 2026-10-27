/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use clap::{Parser, Subcommand};

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

    /// Start of the period (timestamp, date, time or datetime)
    #[arg(short = 'f', long, global = true)]
    from: Option<String>,

    /// End of the period (timestamp, date, time or datetime)
    #[arg(short = 't', long, global = true)]
    to: Option<String>,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Shows all your sessions in a row
    Sessions {
        /// Maximum number of sessions to display.
        #[arg(short = 'L', long)]
        limit: Option<i64>,
    },
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
use iocraft::prelude::*;
use spyland_core::{State, analytics::SessionAnalytics};
use spyland_lib::config::ConfigFile;
use time::{OffsetDateTime, UtcOffset, format_description};

mod config;
mod utils;
use crate::{
    config::{Config, Group},
    utils::*,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config_file = ConfigFile::open_default()?;

    let config: Config = config_file.get_section()?;

    match args.command {
        Command::Sessions { limit } => sessions(args.from, args.to, limit).await,
        Command::Time { ascending, by_time } => {
            time(
                ascending.unwrap_or(config.sort_ascending),
                by_time.unwrap_or(config.sort_by_time),
                args.from,
                args.to,
                config.groups,
            )
            .await
        }
    }
}

async fn sessions(from: Option<String>, to: Option<String>, limit: Option<i64>) -> Result<()> {
    let sessions = load_sessions(from, to, limit).await?;

    let offset = UtcOffset::current_local_offset()?;
    let date_format =
        format_description::parse("[weekday repr:short], [day] [month repr:long] [year]")?;
    let time_format = format_description::parse("[hour]:[minute]")?;

    let mut old_datetime: Option<OffsetDateTime> = None;

    for session in sessions {
        println!("|\n|");

        let datetime = OffsetDateTime::from_unix_timestamp(session.start)?.to_offset(offset);

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

async fn time(
    ascending: bool,
    by_time: bool,
    from: Option<String>,
    to: Option<String>,
    groups: Vec<Group>,
) -> Result<()> {
    let sessions = load_sessions(from, to, None).await?;

    let analytic = SessionAnalytics::new(sessions);

    let time = analytic.time_for_each_app();
    let mut stat: Vec<(State, i64)> = time.into_iter().collect();

    stat.sort_by(|x, y| {
        let cmp = if by_time {
            x.1.cmp(&y.1)
        } else {
            match (&x.0, &y.0) {
                (
                    State::Active {
                        app_id: x,
                        workspace: _,
                    },
                    State::Active {
                        app_id: y,
                        workspace: _,
                    },
                ) => x.cmp(y),

                (_, State::Idle) => std::cmp::Ordering::Less,
                (State::Idle, _) => std::cmp::Ordering::Greater,
            }
        };
        if ascending { cmp } else { cmp.reverse() }
    });

    element! {
        View(flex_direction: FlexDirection::Column) {
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_edges: Some(Edges::Bottom)
            ) {
                #(
                    stat.iter().map(|(state, time)| {
                        let mut group: Option<Group> = None;

                        for grp in &groups {
                            if grp.session_group.matches(state) {
                                group = Some(grp.clone());
                                break;
                            }
                        }

                        element!(
                            View {
                                #(
                                    match group {
                                        Some(g) => {
                                            Some(
                                                element! {
                                                    Text(
                                                        content: format!("[{}] ", &g.name),
                                                        color: g.color
                                                            .as_deref()
                                                            .and_then(parse_hex_color)
                                                            .map(|c| Color::Rgb { r: c.0, g: c.1, b: c.2 })
                                                    )
                                                }
                                            )
                                        }
                                        None => None,
                                    }
                                )
                                Text(content: format!("{}: {}", state, human_duration(*time)))
                            }
                        )
                    })
                )
            }
            Text(content: format!("Total screen time: {}", human_duration(analytic.total_screen_time())))
            Text(content: format!("Idle time: {}", human_duration(analytic.idle_time())))
        }
    }.print();

    Ok(())
}
