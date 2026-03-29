/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::sync::mpsc::Receiver;

pub struct Session {
    utc_start: i64,
    utc_end: i64,

    state: State,
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
