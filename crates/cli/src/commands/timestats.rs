use crate::{
    config::{Alias, Group, Theme},
    utils::{duration_color, human_duration, query_sessions},
};
use anyhow::Result;
use iocraft::prelude::*;
use spyland_core::State;
use std::collections::HashMap;
use time::{OffsetDateTime, UtcOffset, format_description};

pub struct TimeEntry {
    pub state: State,
    pub duration: i64,
    pub group: Option<Group>,
    pub alias: Option<Alias>,
}

pub struct TimeData {
    pub entries: Vec<TimeEntry>,
    pub groups_time: HashMap<Group, i64>,
    pub total_screen_time: i64,
    pub idle_time: i64,
}

pub async fn load_time_data(
    from: i64,
    to: i64,
    ascending: bool,
    by_time: bool,
    display_workspace: bool,
    groups: Vec<Group>,
    aliases: Vec<Alias>,
) -> Result<TimeData> {
    let sessions = query_sessions(from, to, None, aliases).await?;

    let mut entries: HashMap<State, TimeEntry> = HashMap::new();
    let mut groups_time = HashMap::new();
    let mut total_screen_time = 0;
    let mut idle_time = 0;

    for (session, alias) in sessions {
        let duration = session.end - session.start;
        let group = groups
            .iter()
            .find(|g| g.selector.matches(&session.state))
            .cloned();

        if let Some(g) = group.clone() {
            groups_time
                .entry(g)
                .and_modify(|d| *d += duration)
                .or_insert(duration);
        }

        match &session.state {
            State::Active { .. } => {
                total_screen_time += duration;

                let state = if display_workspace {
                    session.state.clone()
                } else {
                    if let State::Active { app_id, .. } = &session.state {
                        State::Active {
                            app_id: app_id.clone(),
                            workspace: None,
                        }
                    } else {
                        session.state
                    }
                };
                entries
                    .entry(state.clone())
                    .and_modify(|entry| entry.duration += duration)
                    .or_insert(TimeEntry {
                        state,
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

    let mut gvec: Vec<(Group, i64)> = groups_time.into_iter().collect();
    gvec.sort_by(|left, right| {
        let ordering = if by_time {
            left.1.cmp(&right.1)
        } else {
            left.0.name.cmp(&right.0.name)
        };

        if ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });

    Ok(TimeData {
        entries,
        groups_time: gvec.into_iter().collect(),
        total_screen_time,
        idle_time,
    })
}

pub fn time(from: i64, to: i64, time_data: TimeData, wdisplay: bool, theme: Theme) -> Result<()> {
    let tst = time_data.total_screen_time;
    let idt = time_data.idle_time;

    let offset = UtcOffset::current_local_offset()?;
    let format = format_description::parse("[day] [month repr:long] [year]")?;

    let fdt = OffsetDateTime::from_unix_timestamp(from)?.to_offset(offset);
    let tdt = OffsetDateTime::from_unix_timestamp(to)?.to_offset(offset);

    let time_period_multiplier = u64::try_from((to - from) / 86400)?.max(1);

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
                padding_bottom: 0,
            ) {
                #(
                    time_data.entries.iter().map(|TimeEntry { state, duration, group, alias }| {
                        element!(
                            View {
                                View(width: 12, padding_right: 1) {
                                    #(
                                        group.as_ref().map(|g| element! {
                                            Text(content: &g.name, color: g.color)
                                        })
                                    )
                                }
                                View(width: 24, padding_right: 1) {
                                    #(
                                        match (state, alias, wdisplay) {
                                            (State::Active { .. }, Some(a), false) =>
                                                Some(element! { View {
                                                    Text(
                                                        content: &a.name,
                                                        color: a.color.unwrap_or(theme.app_name_color),
                                                        weight: Weight::Bold
                                                    )
                                                }}),
                                            (State::Active { workspace: w, .. }, Some(a), true) =>
                                                Some(element! {
                                                    View {
                                                        Text(
                                                            content: &a.name,
                                                            color: a.color.unwrap_or(theme.app_name_color),
                                                            weight: Weight::Bold
                                                        )
                                                        Text(
                                                            content: format!(" ({})",
                                                                w.map(|w| w.to_string()).unwrap_or_default()
                                                            ),
                                                            color: Color::Grey,
                                                        )
                                                    }
                                                }),
                                            (State::Active { app_id, workspace: w, .. }, None, true) =>
                                                Some(element! {
                                                    View {
                                                        Text(
                                                            content: app_id,
                                                            color: theme.app_name_color,
                                                            weight: Weight::Bold
                                                        )
                                                        Text(
                                                            content: format!(" ({})",
                                                                w.map(|w| w.to_string()).unwrap_or_default()
                                                            ),
                                                            color: Color::Grey,
                                                        )
                                                    }
                                                }),
                                            (State::Active { app_id, .. }, None, false) =>
                                                Some(element! { View {
                                                    Text(
                                                        content: app_id,
                                                        color: theme.app_name_color,
                                                        weight: Weight::Bold
                                                    )
                                                }}),
                                            (State::Idle, _, _) => None,
                                        }
                                    )
                                }
                                View(width: 12) {
                                    Text(
                                        content: human_duration(*duration),
                                        color: duration_color(*duration, ((tst) / 3).try_into().unwrap())
                                    )
                                }
                            }
                        )
                    })
                )
            }
            View(
                flex_direction: FlexDirection::Column,
                padding: 1,
            ) {
                #(
                    time_data.groups_time.iter().map(|(group, time)| element! {
                        View {
                            View(width: 12, padding_right: 1) {
                                Text(content: &group.name, color: group.color)
                            }
                            Text(
                                content: human_duration(*time),
                                color: duration_color(*time, ((tst) / 3).try_into().unwrap()),
                            )
                        }
                    })
                )
            }
            View {
                Text(content: "Total screen time: ", weight: Weight::Bold)
                Text(
                    content: human_duration(tst),
                    color: duration_color(tst, time_period_multiplier * theme.timestats_total_time_sensitivity * 60)
                )
            }
            View {
                Text(content: "Idle time: ", color: Color::DarkGrey, weight: Weight::Bold)
                Text(
                    content: human_duration(idt),
                    color: duration_color(idt, time_period_multiplier * theme.timestats_idle_time_sensitivity * 60)
                )
            }
        }
    }.print();

    Ok(())
}
