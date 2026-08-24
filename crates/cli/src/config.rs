/*
 *  spyland-cli — command line interface for spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use serde::{Deserialize, Serialize};
use spyland_core::analytics::SessionGroup;
use spyland_lib::config::ConfigSection;

#[derive(Serialize, Deserialize)]
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

    pub groups: Vec<Group>,
}

impl ConfigSection for Config {
    const SECTION: &'static str = "frontend.cli";
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sort_ascending: true,
            sort_by_time: true,
            groups: Vec::new(),
        }
    }
}
