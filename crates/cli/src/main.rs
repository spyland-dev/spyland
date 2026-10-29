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

    element! {
        View(
            flex_direction: FlexDirection::Column,
            gap: Gap::Length(2),

            border_style: BorderStyle::Round,
            border_edges: Some(Edges::Left),
            border_color: Some(Color::Blue),
        ) {
            #(
                sessions.iter().scan(None::<i64>, |previous, session| {
                    let duration = session.end - session.start;

                    let offset = UtcOffset::current_local_offset().unwrap();
                    let date_format = format_description::parse("[weekday repr:short], [day] [month repr:long] [year]").unwrap();
                    let time_format = format_description::parse("[hour]:[minute]").unwrap();

                    let cdt = OffsetDateTime::from_unix_timestamp(session.start).unwrap().to_offset(offset);
                    let pdt = (*previous).map(|start| OffsetDateTime::from_unix_timestamp(start).unwrap().to_offset(offset));

                    *previous = Some(session.start);

                    Some(element! {
                        View(flex_direction: FlexDirection::Column) {
                            #(
                                if pdt.is_none_or(|p| cdt.date() != p.date()) {
                                    Some(
                                        element! {
                                            View(padding_left: Padding::Length(4), padding_bottom: Padding::Length(2)) {
                                                Text(
                                                    content: cdt.format(&date_format).unwrap(),
                                                    color: Some(Color::Cyan),
                                                    weight: Weight::Bold,
                                                )
                                            }
                                        }
                                    )
                                } else {
                                    None
                                }
                            )
                            View {
                                View(
                                    border_style: BorderStyle::Round,
                                    border_edges: Some(Edges::Top),
                                    border_color: Some(Color::Blue),
                                    margin_right: Margin::Length(1),
                                    width: 3
                                )
                                Text(content: format!("({}) ", cdt.format(&time_format).unwrap()), color: Some(time_color(cdt)))
                                #(
                                    match &session.state {
                                        State::Active { app_id, workspace: _ } => element! {
                                            Text(content: app_id, weight: Weight::Bold, color: Some(Color::DarkCyan))
                                        },
                                        State::Idle => element! {
                                            Text(content: "Idle", color: Color::DarkGrey, weight: Weight::Bold)
                                        }
                                    }
                                )
                                Text(content: ": ")
                                Text(content: human_duration(duration), color: Some(duration_color(duration)))
                            }
                        }
                    })
                })
            )
        }
    }
    .print();

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
