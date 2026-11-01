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
use time::{OffsetDateTime, Time, UtcOffset, format_description};

mod config;
mod utils;
use crate::{
    config::{Config, Group, Theme},
    utils::*,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config_file = ConfigFile::open_default()?;

    let config: Config = config_file.get_section()?;

    let offset = UtcOffset::current_local_offset()?;

    let resolve_range = |from: Option<String>,
                         to: Option<String>,
                         default_lookback_days: i64|
     -> Result<(i64, i64)> {
        let today = OffsetDateTime::now_utc().to_offset(offset).date();
        let from = from
            .map(|from| parse_flexible_time(from, offset))
            .transpose()?
            .unwrap_or_else(|| {
                (today - time::Duration::days(default_lookback_days))
                    .with_time(Time::MIDNIGHT)
                    .assume_offset(offset)
                    .unix_timestamp()
            });
        let to = to
            .map(|to| parse_flexible_time(to, offset))
            .transpose()?
            .unwrap_or_else(|| {
                today
                    .with_time(time::macros::time!(23:59:59))
                    .assume_offset(offset)
                    .unix_timestamp()
            });
        Ok((from, to))
    };

    match args.command {
        Command::Sessions { limit } => {
            let (from, to) = resolve_range(args.from, args.to, 1)?;
            sessions(load_sessions(from, to, limit).await?, config.theme)
        }
        Command::Time { ascending, by_time } => {
            let (from, to) = resolve_range(args.from, args.to, 7)?;
            time(
                ascending.unwrap_or(config.sort_ascending),
                by_time.unwrap_or(config.sort_by_time),
                from,
                to,
                config.groups,
                config.theme,
            )
            .await
        }
    }
}

fn sessions(sessions: Vec<spyland_core::Session>, theme: Theme) -> Result<()> {
    let offset = UtcOffset::current_local_offset()?;
    let date_format =
        format_description::parse("[weekday repr:short], [day] [month repr:long] [year]")?;
    let time_format = format_description::parse("[hour]:[minute]")?;

    element! {
        View(
            flex_direction: FlexDirection::Column,
            gap: Gap::Length(2),

            border_style: BorderStyle::Round,
            border_edges: Edges::Left,
            border_color: theme.timeline_color,
        ) {
            #(
                sessions.iter().scan(None::<i64>, |previous, session| {
                    let duration = session.end - session.start;

                    let cdt = OffsetDateTime::from_unix_timestamp(session.start)
                        .unwrap()
                        .to_offset(offset);
                    let pdt = (*previous)
                        .map(|start| OffsetDateTime::from_unix_timestamp(start)
                        .unwrap()
                        .to_offset(offset));

                    *previous = Some(session.start);

                    Some(element! {
                        View(flex_direction: FlexDirection::Column) {
                            #(
                                if pdt.is_none_or(|p| cdt.date() != p.date()) {
                                    Some(
                                        element! {
                                            View(
                                                padding_left: Padding::Length(4),
                                                padding_bottom: Padding::Length(2)
                                            ) {
                                                Text(
                                                    content: cdt.format(&date_format).unwrap(),
                                                    color: Some(theme.date_color),
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
                                    border_edges: Edges::Top,
                                    border_color: theme.timeline_color,
                                    margin_right: Margin::Length(1),
                                    width: 3
                                )
                                Text(
                                    content: format!("({}) ",
                                    cdt.format(&time_format).unwrap()),
                                    color: time_color(cdt)
                                )
                                #(
                                    match &session.state {
                                        State::Active { app_id, workspace: _ } => element! {
                                            Text(content: app_id, weight: Weight::Bold, color: theme.app_name_color)
                                        },
                                        State::Idle => element! {
                                            Text(
                                                content: "Idle",
                                                color: theme.idle_session_color,
                                                weight: Weight::Bold
                                            )
                                        }
                                    }
                                )
                                Text(content: ": ")
                                Text(
                                    content: human_duration(duration),
                                    color: duration_color(duration, theme.timeline_session_duration_sensitivity * 60)
                                )
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
    from: i64,
    to: i64,
    groups: Vec<Group>,
    theme: Theme,
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

    let tst = analytic.total_screen_time();
    let idt = analytic.idle_time();

    let offset = UtcOffset::current_local_offset()?;
    let format = format_description::parse("[day] [month repr:long] [year]")?;

    let fdt = OffsetDateTime::from_unix_timestamp(from)?.to_offset(offset);
    let tdt = OffsetDateTime::from_unix_timestamp(to)?.to_offset(offset);

    element! {
        View(flex_direction: FlexDirection::Column) {
            View {
                MixedText(contents: vec![
                    MixedTextContent::new("From "),
                    MixedTextContent::new(fdt.format(&format)?)
                        .color(theme.date_color)
                        .weight(Weight::Bold),
                ])
                MixedText(contents: vec![
                    MixedTextContent::new(" to "),
                    MixedTextContent::new(tdt.format(&format)?)
                        .color(theme.date_color)
                        .weight(Weight::Bold),
                ])
            }
            View(
                flex_direction: FlexDirection::Column,
                padding: 1,
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
                                View(width: 12, padding_right: 1) {
                                    #(
                                        match group {
                                            Some(g) => Some(element! {
                                                Text(
                                                    content: g.name,
                                                    color: g.color
                                                        .as_deref()
                                                        .and_then(parse_hex_color)
                                                        .map(|c| Color::Rgb { r: c.0, g: c.1, b: c.2 }),
                                                    italic: true,
                                                )
                                            }),
                                            None => None,
                                        }
                                    )
                                }
                                View(width: 24, padding_right: 1) {
                                    #(
                                        if let State::Active { app_id, workspace: _ } = state {
                                            Some(
                                                element! {
                                                    Text(
                                                        content: app_id,
                                                        color: theme.app_name_color,
                                                        weight: Weight::Bold
                                                    )
                                                }
                                            )
                                        } else { None }
                                    )
                                }
                                View(width: 12) {
                                    Text(
                                        content: human_duration(*time),
                                        color: duration_color(*time, ((tst) / 3).try_into().unwrap())
                                    )
                                }
                            }
                        )
                    })
                )
            }
            View {
                Text(content: "Total screen time: ", weight: Weight::Bold)
                Text(
                    content: human_duration(tst),
                    color: duration_color(tst, theme.timestats_total_time_sensitivity * 60)
                )
            }
            View {
                Text(content: "Idle time: ", color: Color::DarkGrey, weight: Weight::Bold)
                Text(
                    content: human_duration(idt),
                    color: duration_color(idt, theme.timestats_idle_time_sensitivity * 60)
                )
            }
        }
    }.print();

    Ok(())
}
