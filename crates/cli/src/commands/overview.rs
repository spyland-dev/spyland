use crate::{
    commands::timestats::{TimeData, TimeEntry},
    config::Theme,
    utils::{duration_color, human_duration},
};
use anyhow::Result;
use iocraft::prelude::*;
use spyland_core::State;
use time::{OffsetDateTime, UtcOffset, format_description};

pub fn overview(from: i64, to: i64, time_data: TimeData, theme: Theme) -> Result<()> {
    let offset = UtcOffset::current_local_offset()?;
    let format = format_description::parse("[day] [month repr:long] [year]")?;

    let fdt = OffsetDateTime::from_unix_timestamp(from)?.to_offset(offset);
    let tdt = OffsetDateTime::from_unix_timestamp(to)?.to_offset(offset);

    element! {
        View(
            width: 100pct,
            flex_direction: FlexDirection::Column,
            gap: 1,
            padding_left: 3,
            padding_right: 3
        ) {
            MixedText(contents: vec![
                MixedTextContent::new("From "),
                MixedTextContent::new(fdt.format(&format)?)
                    .color(theme.date_color)
                    .weight(Weight::Bold),
                MixedTextContent::new(" to "),
                MixedTextContent::new(tdt.format(&format)?)
                    .color(theme.date_color)
                    .weight(Weight::Bold),
            ], align: TextAlign::Center)

            View(flex_grow: 1.0) {
                #(
                    time_data.entries.iter().map(|TimeEntry { state, duration, group, alias }| {
                        let percent = (*duration as f32) / (time_data.total_screen_time as f32) * 100.0;
                        let color = match (alias, group) {
                            (Some(a), _) => a.color,
                            (None, Some(g)) => g.color,
                            (None, None) => None,
                        }.unwrap_or(Color::Grey);
                        let name = if percent >= 7.5 {
                            match (state, alias) {
                                (State::Active { app_id, .. }, None) => app_id,
                                (State::Active { .. }, Some(a)) => &a.name,
                                (_, _) => "",
                            }
                        } else { "" };

                        element! {
                            View(
                                width: Size::Percent(percent),
                                background_color: color,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                            ) {
                                Text(
                                    content: name,
                                    color: Color::Black,
                                    weight: Weight::Bold,
                                    align: TextAlign::Center,
                                )
                            }
                        }
                    })
                )
            }

            View(flex_grow: 1.0) {
                View(flex_direction: FlexDirection::Column) {
                    Text(content: "Most used applications:")
                    #(
                        time_data.entries
                            .iter()
                            .take(5)
                            .enumerate()
                            .map(
                                |(index, TimeEntry { state, duration, alias, .. })| {
                                    let index = index + 1;
                                    let name = match (state, alias) {
                                        (State::Active { app_id, .. }, None) => app_id,
                                        (State::Active { .. }, Some(a)) => &a.name,
                                        (_, _) => "",
                                    };

                                    element! {
                                        MixedText(contents: vec![
                                            MixedTextContent::new(format!("{index}. ")),
                                            MixedTextContent::new(name)
                                                .color(alias
                                                    .clone()
                                                    .and_then(|a| a.color)
                                                    .unwrap_or(theme.app_name_color)
                                                )
                                                .weight(Weight::Bold),
                                            MixedTextContent::new(format!(" ({})", human_duration(*duration)))
                                                .color(
                                                    duration_color(
                                                        *duration,
                                                        theme.timestats_total_time_sensitivity * 60
                                                    )
                                                )
                                            ]
                                        )
                                    }
                                }
                            )
                    )
                }

                View(flex_direction: FlexDirection::Column, flex_grow: 1.0, align_items: AlignItems::End) {
                    Text(content: "Top groups time:")
                    #(
                        time_data.groups_time
                            .iter()
                            .take(5)
                            .enumerate()
                            .map(
                                |(index, (group, duration))| {
                                    let index = index + 1;

                                    element! {
                                        MixedText(contents: vec![
                                            MixedTextContent::new(format!("{index}. ")),
                                            MixedTextContent::new(&group.name)
                                                .color(group.color.unwrap_or(Color::Grey))
                                                .weight(Weight::Bold),
                                            MixedTextContent::new(format!(" ({})", human_duration(*duration)))
                                                .color(
                                                    duration_color(
                                                        *duration,
                                                        theme.timestats_total_time_sensitivity * 60
                                                    )
                                                )
                                            ]
                                        )
                                    }
                                }
                            )
                    )
                }
            }
        }
    }.print();

    Ok(())
}
