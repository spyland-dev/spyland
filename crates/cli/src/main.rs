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
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Shows all your sessions in a row
    Sessions {
        /// Start of the period (timestamp, date, time or datetime)
        #[arg(short = 'f', long)]
        from: Option<String>,
        /// End of the period (timestamp, date, time or datetime)
        #[arg(short = 't', long)]
        to: Option<String>,
        /// Maximum number of sessions to display.
        #[arg(short = 'L', long)]
        limit: Option<i64>,
        /// Enable workspace displaying
        #[arg(short = 'W', long)]
        workspace: Option<bool>,
    },
    /// Shows your total screen time
    Time {
        /// Start of the period (timestamp, date, time or datetime)
        #[arg(short = 'f', long)]
        from: Option<String>,
        /// End of the period (timestamp, date, time or datetime)
        #[arg(short = 't', long)]
        to: Option<String>,
        /// Sort ascending
        #[arg(short = 'A', long)]
        ascending: Option<bool>,
        /// Sort by time
        #[arg(short = 'T', long)]
        by_time: Option<bool>,
        /// Enable workspace displaying
        #[arg(short = 'W', long)]
        workspace: Option<bool>,
    },
    /// Shows a simple visual overview of your activity
    Overview {
        /// From what date to show overview (default: today)
        date: Option<String>,

        /// How to display your activity. Available options: daily, weekly, monthly
        #[arg(short = 'd', long = "division", default_value = TimeDivision::Daily)]
        time_division: TimeDivision,
    },
}

use anyhow::Result;
use spyland_lib::config::ConfigFile;
use time::{OffsetDateTime, Time, UtcOffset, macros::time};

mod commands;
mod config;
mod utils;
use crate::{
    commands::{
        overview::{TimeDivision, divide_time},
        timeline::load_sessions,
        timestats::load_time_data,
    },
    config::*,
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
                    .with_time(time!(23:59:59))
                    .assume_offset(offset)
                    .unix_timestamp()
            });
        Ok((from, to))
    };

    match args.command {
        Command::Sessions {
            from,
            to,
            limit,
            workspace,
        } => {
            let (from, to) = resolve_range(from, to, 1)?;
            commands::timeline::sessions(
                load_sessions(from, to, limit, config.aliases).await?,
                workspace.unwrap_or(config.display_workspace),
                config.theme,
            )
        }
        Command::Time {
            from,
            to,
            ascending,
            by_time,
            workspace,
        } => {
            let (from, to) = resolve_range(from, to, 7)?;
            commands::timestats::time(
                from,
                to,
                load_time_data(
                    from,
                    to,
                    ascending.unwrap_or(config.sort_ascending),
                    by_time.unwrap_or(config.sort_by_time),
                    workspace.unwrap_or(config.display_workspace),
                    config.groups,
                    config.aliases,
                )
                .await?,
                workspace.unwrap_or(config.display_workspace),
                config.theme,
            )
        }
        Command::Overview {
            date,
            time_division,
        } => {
            use time::{Date, Month, Weekday};

            let target = match date {
                Some(d) => OffsetDateTime::from_unix_timestamp(parse_flexible_time(d, offset)?)?,
                None => OffsetDateTime::now_utc(),
            }
            .to_offset(offset)
            .date();

            let (from, to) = match time_division {
                TimeDivision::Daily => (
                    Date::from_iso_week_date(target.year(), target.iso_week(), Weekday::Monday)?,
                    Date::from_iso_week_date(target.year(), target.iso_week(), Weekday::Sunday)?,
                ),
                TimeDivision::Weekly => (
                    target.replace_day(1)?,
                    // The last day of the current month.
                    Date::from_calendar_date(target.year(), target.month().next(), 1)?
                        .previous_day()
                        .unwrap(),
                ),
                TimeDivision::Monthly => (
                    Date::from_calendar_date(target.year(), Month::January, 1)?,
                    target,
                ),
            };

            let from = from
                .with_time(Time::MIDNIGHT)
                .assume_offset(offset)
                .unix_timestamp();
            let to = to
                .with_time(time!(23:59:59))
                .assume_offset(offset)
                .unix_timestamp();

            commands::overview::overview(
                from,
                to,
                load_time_data(from, to, false, true, false, config.groups, config.aliases).await?,
                divide_time(from, to, time_division.clone()).await?,
                time_division,
                config.theme,
            )
        }
    }
}
