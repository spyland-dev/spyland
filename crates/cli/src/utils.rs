/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::{Alias, Group};
use anyhow::Result;
use iocraft::prelude::Color;
use spyland_core::{Session, State};
use spyland_lib::db::{Db, QueryFilter};
use std::collections::HashMap;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, format_description};

pub fn duration_color(seconds: i64, sensitivity: u64) -> Color {
    assert_ne!(sensitivity, 0, "`sensitivity` can not be zero.");
    let seconds: u64 = seconds
        .try_into()
        .expect("`seconds` can not be less than zero.");

    let seconds = seconds.clamp(0, sensitivity * 2);
    let (r, g) = if seconds <= sensitivity {
        ((seconds * 255 / sensitivity) as u8, 255)
    } else {
        (
            255,
            (255 - (seconds - sensitivity) * 255 / sensitivity) as u8,
        )
    };

    Color::Rgb { r, g, b: 0 }
}

pub fn time_color(time: OffsetDateTime) -> Color {
    const KEYFRAMES: [(u16, (u8, u8, u8)); 7] = [
        (0, (78, 101, 151)),
        (7 * 60, (222, 202, 141)),
        (12 * 60, (232, 231, 160)),
        (15 * 60, (224, 185, 73)),
        (18 * 60, (242, 124, 75)),
        (20 * 60, (91, 111, 164)),
        (24 * 60, (78, 101, 151)),
    ];

    let minute = u16::from(time.hour()) * 60 + u16::from(time.minute());
    let interval = KEYFRAMES
        .windows(2)
        .find(|pair| minute >= pair[0].0 && minute <= pair[1].0)
        .unwrap();

    let (start_minute, (start_r, start_g, start_b)) = interval[0];
    let (end_minute, (end_r, end_g, end_b)) = interval[1];

    let elapsed = i32::from(minute - start_minute);
    let duration = i32::from(end_minute - start_minute);

    let interpolate = |start: u8, end: u8| {
        let start = i32::from(start);
        let difference = i32::from(end) - start;
        (start + difference * elapsed / duration) as u8
    };

    Color::Rgb {
        r: interpolate(start_r, end_r),
        g: interpolate(start_g, end_g),
        b: interpolate(start_b, end_b),
    }
}

pub fn parse_flexible_time(s: String, offset: UtcOffset) -> Result<i64> {
    let s = s.trim();

    if let Ok(timestamp) = s.parse::<i64>() {
        return Ok(timestamp);
    }

    if let Ok(odt) = OffsetDateTime::parse(s, &format_description::well_known::Rfc3339) {
        return Ok(odt.unix_timestamp());
    }

    let normalized = s.replace(['+', '_', 'T'], " ");

    let date_formats = [
        "[year]-[month]-[day]",
        "[year]/[month]/[day]",
        "[year].[month].[day]",
        "[month]-[day]-[year]",
        "[month]/[day]/[year]",
    ];

    let time_formats = [
        "[hour]:[minute]",
        "[hour]:[minute]:[second]",
        "[hour repr:12]:[minute] [period]",
        "[hour repr:12]:[minute]:[second] [period]",
    ];

    for date_fmt in date_formats {
        for time_fmt in time_formats {
            let fmt_string = format!("{date_fmt} {time_fmt}");
            if let Ok(format) = format_description::parse(&fmt_string)
                && let Ok(pdt) = PrimitiveDateTime::parse(&normalized, &format)
            {
                return Ok(pdt.assume_offset(offset).unix_timestamp());
            }
        }
    }

    for date_fmt in date_formats {
        if let Ok(format) = format_description::parse(date_fmt)
            && let Ok(date) = Date::parse(s, &format)
        {
            let pdt = PrimitiveDateTime::new(date, Time::MIDNIGHT);
            return Ok(pdt.assume_offset(offset).unix_timestamp());
        }
    }

    for time_fmt in time_formats {
        if let Ok(format) = format_description::parse(time_fmt)
            && let Ok(time) = Time::parse(s, &format)
        {
            let today = OffsetDateTime::now_utc().to_offset(offset).date();
            let pdt = PrimitiveDateTime::new(today, time);
            return Ok(pdt.assume_offset(offset).unix_timestamp());
        }
    }

    anyhow::bail!("Unsupported date/time format: '{s}'")
}

pub fn human_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    match (hours, minutes, seconds) {
        (0, 0, 0) => "0s".to_owned(),
        (0, 0, s) => format!("{s}s"),
        (0, m, s) => format!("{m}m {s}s"),
        (h, 0, 0) => format!("{h}h"),
        (h, m, 0) => format!("{h}h {m}m"),
        (h, m, s) => format!("{h}h {m}m {s}s"),
    }
}

pub type SessionEntry = (Session, Option<Alias>);

pub async fn load_sessions(
    from: i64,
    to: i64,
    limit: Option<i64>,
    aliases: Vec<Alias>,
) -> Result<Vec<SessionEntry>> {
    let db = Db::open_default().await?;

    let sessions_sql = match (from, to, limit) {
        (0, 0, None) => db.query_all().await?,
        (f, t, limit) => {
            db.query_filtered(QueryFilter {
                from: Some(f),
                to: Some(t),
                limit,
                ..Default::default()
            })
            .await?
        }
    };

    Ok(sessions_sql
        .into_iter()
        .map(Session::from)
        .map(|session| {
            let alias = match &session.state {
                State::Active { app_id, .. } => aliases
                    .iter()
                    .find(|alias| alias.app_ids.contains(app_id))
                    .cloned(),
                State::Idle => None,
            };
            (session, alias)
        })
        .collect())
}

pub struct TimeEntry {
    pub state: State,
    pub duration: i64,
    pub group: Option<Group>,
    pub alias: Option<Alias>,
}

pub struct TimeData {
    pub entries: Vec<TimeEntry>,
    pub total_screen_time: i64,
    pub idle_time: i64,
}

pub async fn load_time_data(
    from: i64,
    to: i64,
    ascending: bool,
    by_time: bool,
    groups: Vec<Group>,
    aliases: Vec<Alias>,
) -> Result<TimeData> {
    let sessions = load_sessions(from, to, None, aliases).await?;

    let mut entries: HashMap<State, TimeEntry> = HashMap::new();
    let mut total_screen_time = 0;
    let mut idle_time = 0;

    for (session, alias) in sessions {
        let duration = session.end - session.start;
        let group = groups
            .iter()
            .find(|g| g.selector.matches(&session.state))
            .cloned();

        match &session.state {
            State::Active { .. } => {
                total_screen_time += duration;

                entries
                    .entry(session.state.clone())
                    .and_modify(|entry| entry.duration += duration)
                    .or_insert(TimeEntry {
                        state: session.state,
                        duration,
                        group,
                        alias,
                    });
            }
            State::Idle => idle_time += duration,
        }
    }

    let mut entries: Vec<TimeEntry> = entries.into_values().collect();
    entries.sort_by(|left, right| {
        let ordering = if by_time {
            left.duration.cmp(&right.duration)
        } else {
            match (&left.state, &right.state) {
                (State::Active { app_id: left, .. }, State::Active { app_id: right, .. }) => {
                    left.cmp(right)
                }
                _ => std::cmp::Ordering::Equal,
            }
        };

        if ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });

    Ok(TimeData {
        entries,
        total_screen_time,
        idle_time,
    })
}
