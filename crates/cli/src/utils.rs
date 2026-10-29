/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use iocraft::prelude::Color;
use spyland_core::Session;
use spyland_lib::db::{Db, QueryFilter};
use std::fmt::Write;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, format_description};

pub fn duration_color(seconds: i64) -> Color {
    const SENSITIVITY: i64 = 30 * 60;

    let seconds = seconds.clamp(0, SENSITIVITY * 2);
    let (r, g) = if seconds <= SENSITIVITY {
        ((seconds * 255 / SENSITIVITY) as u8, 255)
    } else {
        (
            255,
            (255 - (seconds - SENSITIVITY) * 255 / SENSITIVITY) as u8,
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

pub fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let hex = s.trim().trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

pub async fn load_sessions(
    from: Option<String>,
    to: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<Session>> {
    let db = Db::open_default().await?;
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);

    let sessions_sql = match (from, to, limit) {
        (None, None, None) => db.query_all().await?,
        (f, t, limit) => {
            let from = match f {
                Some(s) => Some(parse_flexible_time(s, offset)?),
                None => None,
            };
            let to = match t {
                Some(s) => Some(parse_flexible_time(s, offset)?),
                None => None,
            };
            db.query_filtered(QueryFilter {
                from,
                to,
                limit,
                ..Default::default()
            })
            .await?
        }
    };

    Ok(sessions_sql.into_iter().map(Session::from).collect())
}
