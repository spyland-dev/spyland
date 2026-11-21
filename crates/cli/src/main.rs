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
        /// Enable workspace displaying
        #[arg(short = 'W', long)]
        workspace: Option<bool>,
    },
    /// Shows your total screen time
    Time {
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
    Overview,
}

use anyhow::Result;
use spyland_lib::config::ConfigFile;
use time::{OffsetDateTime, Time, UtcOffset};

mod commands;
mod config;
mod utils;
use crate::{
    commands::{timeline::load_sessions, timestats::load_time_data},
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
                    .with_time(time::macros::time!(23:59:59))
                    .assume_offset(offset)
                    .unix_timestamp()
            });
        Ok((from, to))
    };

    match args.command {
        Command::Sessions { limit, workspace } => {
            let (from, to) = resolve_range(args.from, args.to, 1)?;
            commands::timeline::sessions(
                load_sessions(
                    from,
                    to,
                    limit,
                    config.aliases,
                    workspace.unwrap_or(config.display_workspace),
                )
                .await?,
                workspace.unwrap_or(config.display_workspace),
                config.theme,
            )
        }
        Command::Time {
            ascending,
            by_time,
            workspace,
        } => {
            let (from, to) = resolve_range(args.from, args.to, 7)?;
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
        Command::Overview => {
            let (from, to) = resolve_range(args.from, args.to, 7)?;

            commands::overview::overview(
                from,
                to,
                load_time_data(from, to, false, true, false, config.groups, config.aliases).await?,
                config.theme,
            )
        }
    }
}
