use self::TimeDivision::{Daily, Monthly, Weekly};
use crate::{
    commands::timestats::{TimeData, TimeEntry},
    config::Theme,
    utils::{duration_color, human_duration, query_sessions},
};
use anyhow::Result;
use iocraft::prelude::*;
use spyland_core::State;
use time::{OffsetDateTime, Time, UtcOffset, ext::NumericalDuration, format_description};

pub struct DividedTimeData {
    pub start: i64,
    pub total_screen_time: i64,
    pub idle_time: i64,
}

#[derive(Clone)]
pub enum TimeDivision {
    Daily,
    Weekly,
    Monthly,
}

impl TimeDivision {
    pub fn advanced_date_time(&self, dt: OffsetDateTime) -> Option<OffsetDateTime> {
        match &self {
            Daily => dt.checked_add(1.days()),
            Weekly => dt.checked_add(1.weeks()),
            Monthly => {
                use time::Month;

                let month = dt.month();

                if let Month::December = month {
                    dt.replace_year(dt.year() + 1)
                        .ok()?
                        .replace_month(Month::January)
                } else {
                    dt.replace_month(month.next())
                }
                .ok()
            }
        }
    }
}

impl std::str::FromStr for TimeDivision {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "daily" => Ok(Daily),
            "1" => Ok(Daily),
            "weekly" => Ok(Weekly),
            "7" => Ok(Weekly),
            "monthly" => Ok(Monthly),
            "30" => Ok(Monthly),
            _ => anyhow::bail!("Unrecognized format"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<clap::builder::OsStr> for TimeDivision {
    fn into(self) -> clap::builder::OsStr {
        match self {
            Daily => "daily",
            Weekly => "weekly",
            Monthly => "monthly",
        }
        .into()
    }
}

pub async fn divide_time(
    from: i64,
    to: i64,
    time_division: TimeDivision,
) -> Result<Vec<DividedTimeData>> {
    let offset = UtcOffset::current_local_offset()?;

    let date = OffsetDateTime::from_unix_timestamp(from)?
        .to_offset(offset)
        .date();
    let mut dtd = Vec::new();

    let mut start = date.with_time(Time::MIDNIGHT).assume_offset(offset);
    let mut end = start;

    loop {
        end = time_division.advanced_date_time(end).unwrap();

        let s = start.unix_timestamp();
        let e = end.unix_timestamp();

        if s > to {
            break;
        }

        let sessions = query_sessions(s, e, None).await?;

        let mut total_screen_time = 0;
        let mut idle_time = 0;

        for session in &sessions {
            match session.state {
                State::Active { .. } => total_screen_time += session.end - session.start,
                State::Idle => idle_time += session.end - session.start,
            }
        }

        if total_screen_time > 0 {
            dtd.push(DividedTimeData {
                start: s,
                total_screen_time,
                idle_time,
            });
        }

        start = time_division.advanced_date_time(start).unwrap();
    }

    Ok(dtd)
}

pub fn overview(
    from: i64,
    to: i64,
    time_data: TimeData,
    dtd: Vec<DividedTimeData>,
    time_division: TimeDivision,
    theme: Theme,
) -> Result<()> {
    let offset = UtcOffset::current_local_offset()?;
    let format = format_description::parse("[day] [month repr:long] [year]")?;
    let short_format = match time_division {
        Daily => format_description::parse("[weekday repr:short] [day] [month repr:short]"),
        Weekly => format_description::parse("[week_number]. [day] [month repr:short]"),
        Monthly => format_description::parse("[month repr:long]"),
    }?;

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
                    dtd.into_iter().map(|DividedTimeData { start, total_screen_time, idle_time }| {
                        let divisor = match time_division {
                            Daily => 1,
                            Weekly => 3,
                            Monthly => 6,
                        };
                        let tst_height = total_screen_time / 60 / 60 / divisor;
                        let idt_height = idle_time / 60 / 60 / divisor;

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
                                    min_height: 1,
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
