/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::sync::mpsc::Receiver;

#[derive(Clone)]
pub struct Session {
    pub utc_start: i64,
    pub utc_end: i64,

    pub state: State,
}

impl Session {
    pub fn new_empty() -> Self {
        Self {
            utc_start: 0,
            utc_end: 0,
            state: State::Idle,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.utc_start <= 0 && self.utc_end <= 0
    }
}
pub enum State {
    Active {
        app_id: String,
        workspace: i32,
        // activity: ???,
    },
    Idle,
}

pub enum Event {
    ActiveWindowChanged(Option<String>),
    WorkspaceChanged(i32),
    Idle(bool),
}

pub trait Backend {
    fn is_available() -> bool;
    fn subscribe(&mut self) -> Receiver<Event>;
}
