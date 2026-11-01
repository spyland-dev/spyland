/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use iocraft::Color;
use serde::{Deserialize, Serialize};
use spyland_core::analytics::SessionGroup;
use spyland_lib::config::ConfigSection;

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    // pub icon: String,
    pub color: Option<String>,

    #[serde(flatten)]
    pub session_group: SessionGroup,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub sort_ascending: bool,
    pub sort_by_time: bool,

    pub theme: Theme,
    pub groups: Vec<Group>,
}

impl ConfigSection for Config {
    const SECTION: &'static str = "frontend.cli";
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub timeline_color: Color,
    pub date_color: Color,

    pub app_name_color: Color,
    pub idle_session_color: Color,

    pub timeline_session_duration_sensitivity: u64,
    pub timestats_total_time_sensitivity: u64,
    pub timestats_idle_time_sensitivity: u64,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            timeline_color: Color::Blue,
            date_color: Color::Cyan,
            app_name_color: Color::DarkCyan,
            idle_session_color: Color::DarkGrey,
            timeline_session_duration_sensitivity: 30,
            timestats_total_time_sensitivity: 6 * 60,
            timestats_idle_time_sensitivity: 4 * 60,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            sort_ascending: true,
            sort_by_time: true,
            theme: Theme::default(),
            groups: Vec::new(),
        }
    }
}
