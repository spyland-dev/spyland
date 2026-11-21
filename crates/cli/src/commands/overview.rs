use crate::{
    commands::timestats::{TimeData, TimeEntry},
    config::Theme,
    utils::{duration_color, human_duration, query_sessions},
};
use anyhow::Result;
use iocraft::prelude::*;
use spyland_core::State;
use time::{OffsetDateTime, Time, UtcOffset, format_description, macros::time};

pub struct DividedTimeData {
    pub start: i64,
    pub total_screen_time: i64,
    pub idle_time: i64,
}

pub async fn divide_time(from: i64, to: i64) -> Result<Vec<DividedTimeData>> {
    let offset = UtcOffset::current_local_offset()?;

    let mut date = OffsetDateTime::from_unix_timestamp(from)?
        .to_offset(offset)
        .date();
    let mut dtd = Vec::new();

    loop {
        date = date.next_day().unwrap();
        let start = date
            .with_time(Time::MIDNIGHT)
            .assume_offset(offset)
            .unix_timestamp();
        let end = date
            .with_time(time!(23:59:59))
            .assume_offset(offset)
            .unix_timestamp();

        if start > to {
            break;
        }

        let sessions = query_sessions(start, end, None).await?;

        if let Some(first) = sessions.first() {
            let mut total_screen_time = 0;
            let mut idle_time = 0;

            for session in &sessions {
                match session.state {
                    State::Active { .. } => total_screen_time += session.end - session.start,
                    State::Idle => idle_time += session.end - session.start,
                }
            }

            dtd.push(DividedTimeData {
                start: first.start,
                total_screen_time,
                idle_time,
            });
        }
    }

    Ok(dtd)
}

pub fn overview(
    from: i64,
    to: i64,
    time_data: TimeData,
    dtd: Vec<DividedTimeData>,
    theme: Theme,
) -> Result<()> {
    let offset = UtcOffset::current_local_offset()?;
    let format = format_description::parse("[day] [month repr:long] [year]")?;
    let short_format = format_description::parse("[weekday repr:short] [day] [month repr:short]")?;

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

            View(flex_grow: 1.0, align_items: AlignItems::Center) {
                #(
                    dtd.into_iter().take(7).map(|DividedTimeData { start, total_screen_time, idle_time }| {
                        let tst_height = total_screen_time / 60 / 60;
                        let idt_height = idle_time / 60 / 60;

                        let date = OffsetDateTime::from_unix_timestamp(start)
                            .unwrap()
                            .to_offset(offset)
                            .format(&short_format)
                            .unwrap();

                        element! {
                            View(
                                flex_grow: 1.0,
                                height: 100pct,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::FlexEnd,
                            ) {
                                View(
                                    height: Size::Length(tst_height as u32),
                                    width: 15pct,
                                    background_color: Color::Blue
                                )
                                View(
                                    height: Size::Length(idt_height as u32),
                                    width: 15pct,
                                    background_color: Color::DarkGrey
                                )

                                Text(content: date, weight: Weight::Bold)
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
