/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

mod analytics;
pub use analytics::SessionAnalytics;

pub mod manager;

#[derive(Clone)]
pub struct Session {
    pub utc_start: u64,
    pub utc_end: u64,

    pub state: State,
}

impl Session {
    pub fn new_empty() -> Self {
        Self {
            utc_start: 0,
            utc_end: 0,
            state: State::Empty,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.utc_start <= 0 && self.utc_end <= 0 && self.state == State::Empty
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Active {
        app_id: String,
        workspace: Option<i32>,
        // activity: ???,
    },
    Idle,
    Empty,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    ActiveWindowChanged(Option<String>),
    WorkspaceChanged(i32),
    Idle(bool),
    Tick,
}

use std::sync::mpsc::Receiver;
pub trait Backend {
    fn is_available(&self) -> bool;
    fn subscribe(&mut self) -> Receiver<Event>;
}
