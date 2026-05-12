/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::{Session, State};
use std::collections::HashMap;

pub struct SessionAnalytics {
    sessions: Vec<Session>,
}

impl SessionAnalytics {
    pub fn new(sessions: Vec<Session>) -> Self {
        Self { sessions }
    }

    pub fn total_screen_time(&self) -> u64 {
        let mut counter: u64 = 0;

        for s in &self.sessions {
            counter += s.utc_end - s.utc_start;
        }

        counter
    }

    pub fn screen_time_app(&self, target_app_id: String) -> u64 {
        let mut counter: u64 = 0;

        for s in &self.sessions {
            if let State::Active { app_id, .. } = &s.state {
                if *app_id == target_app_id {
                    counter += s.utc_end - s.utc_start;
                }
            }
        }

        counter
    }

    pub fn idle_time(&self) -> u64 {
        let mut counter: u64 = 0;

        for s in &self.sessions {
            if let State::Idle = &s.state {
                counter += s.utc_end - s.utc_start;
            }
        }

        counter
    }

    pub fn time_for_each_app(&self) -> HashMap<String, u64> {
        let mut hash_map = HashMap::new();

        for s in &self.sessions {
            if let State::Active { app_id, .. } = &s.state {
                let duration = s.utc_end - s.utc_start;

                hash_map
                    .entry(app_id.to_owned())
                    .and_modify(|v| *v += duration)
                    .or_insert(duration);
            }
        }

        hash_map
    }
}
