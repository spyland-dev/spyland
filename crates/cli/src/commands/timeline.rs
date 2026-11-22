/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::{
    config::{Alias, Theme},
    utils::{duration_color, human_duration, query_sessions, time_color},
};
use anyhow::Result;
use iocraft::prelude::*;
use spyland_core::Session;
use spyland_core::State;
use time::{OffsetDateTime, UtcOffset, format_description};

pub type SessionEntry = (Session, Option<Alias>);

pub async fn load_sessions(
    from: i64,
    to: i64,
    limit: Option<i64>,
    aliases: Vec<Alias>,
) -> Result<Vec<SessionEntry>> {
    let sessions = query_sessions(from, to, limit)
        .await?
        .into_iter()
        .map(|session| {
            let alias = if let State::Active { app_id, .. } = &session.state {
                aliases.iter().find(|a| a.app_ids.contains(app_id)).cloned()
            } else {
                None
            };

            (session, alias)
        })
        .collect();

    Ok(sessions)
}

pub fn sessions(sessions: Vec<SessionEntry>, wdisplay: bool, theme: Theme) -> Result<()> {
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
                sessions.iter().scan(None::<i64>, |previous, (session, alias)| {
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
                                    match (&session.state, alias, wdisplay) {
                                        (State::Active { .. }, Some(al), false) => element!( View {
                                            Text(
                                                content: &al.name,
                                                weight: Weight::Bold,
                                                color: al.color.unwrap_or(theme.app_name_color)
                                            )
                                        }),
                                        (State::Active { workspace: w, .. }, Some(al), true) => element!(
                                            View {
                                                Text(
                                                    content: &al.name,
                                                    weight: Weight::Bold,
                                                    color: al.color.unwrap_or(theme.app_name_color)
                                                )
                                                Text(
                                                    content: format!(" ({})",
                                                        w.map(|w| w.to_string()).unwrap_or_default()
                                                    ),
                                                    color: Color::Grey,
                                                )
                                            }),
                                        (State::Active { app_id, workspace: w, .. }, None, true) => element!(
                                            View {
                                                Text(
                                                    content: app_id,
                                                    weight: Weight::Bold,
                                                    color: theme.app_name_color
                                                )
                                                Text(
                                                    content: format!(" ({})",
                                                        w.map(|w| w.to_string()).unwrap_or_default()
                                                    ),
                                                    color: Color::Grey,
                                                )
                                            }),
                                        (State::Active { app_id, .. }, None, false) => element! ( View {
                                            Text(
                                                content: app_id,
                                                weight: Weight::Bold,
                                                color: theme.app_name_color
                                            )
                                        }),
                                        (State::Idle, _, _) => element!( View {
                                            Text(
                                                content: "Idle",
                                                color: theme.idle_session_color,
                                                weight: Weight::Bold
                                            )
                                        })
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
