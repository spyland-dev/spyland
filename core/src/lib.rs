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

pub struct SessionManager<C: Clock> {
    current: Session,
    workspace: i32,
    clock: C,
    sessions: Vec<Session>,
}

impl<C: Clock> SessionManager<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            workspace: -1,
            current: Session::new_empty(),
            sessions: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::ActiveWindowChanged(a) => {
                let Some(app_id) = a else {
                    return;
                };

                self.new_session();

                let now = self.clock.now();
                self.current.utc_start = now;
                self.current.state = State::Active {
                    app_id: app_id,
                    workspace: self.workspace,
                };
            }
            Event::WorkspaceChanged(id) => {
                self.workspace = id;
            }
            Event::Idle(_) => {
                // TODO:
            }
            Event::Tick => {
                self.current.utc_end = self.clock.now();
            }
        }
    }

    fn new_session(&mut self) {
        if !self.current.is_empty() {
            self.sessions.push(self.current.clone());
            self.current = Session::new_empty();
        }
    }

    pub fn sessions(&self) -> &Vec<Session> {
        &self.sessions
    }
}

pub trait Clock {
    fn now(&self) -> i64;
}

#[derive(Clone)]
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
    Tick,
}

pub trait Backend {
    fn is_available() -> bool;
    fn subscribe(&mut self) -> Receiver<Event>;
}
