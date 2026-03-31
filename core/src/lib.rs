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
            state: State::Empty,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.utc_start <= 0 && self.utc_end <= 0 && self.state == State::Empty
    }
}

pub struct SessionManager<C: Clock> {
    current: Session,
    workspace: i32,
    clock: C,
    sessions: Vec<Session>,
    last_flush: i64,
}

pub const SESSION_MANAGER_FLUSH_INTERVAL: i64 = 15;

impl<C: Clock> SessionManager<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            workspace: -1,
            current: Session::new_empty(),
            sessions: Vec::new(),
            last_flush: 0,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::ActiveWindowChanged(a) => {
                let Some(app_id) = a else {
                    return;
                };

                self.new_session();

                self.current.state = State::Active {
                    app_id,
                    workspace: self.workspace,
                }
            }
            Event::WorkspaceChanged(id) => {
                self.workspace = id;
            }
            Event::Idle(_) => {
                // TODO:
            }
            Event::Tick => {
                let now = self.clock.now();

                self.update();

                if now - self.last_flush >= SESSION_MANAGER_FLUSH_INTERVAL {
                    self.flush();

                    self.last_flush = now;
                }
            }
        }
    }

    fn new_session(&mut self) {
        let now = self.clock.now();

        if !self.current.is_empty() {
            self.current.utc_end = now;
            self.sessions.push(self.current.clone());
        }

        self.current = Session::new_empty();
        self.current.utc_start = now;
    }

    pub fn update(&mut self) {
        if self.current.is_empty() {
            return;
        }

        self.current.utc_end = self.clock.now();
    }

    pub fn flush(&mut self) {
        if self.current.is_empty() {
            return;
        }

        let current = self.current.clone();

        if let Some(last) = self.sessions.last_mut() {
            if last.state == current.state {
                last.utc_end = current.utc_end;
                return;
            }
        }

        self.sessions.push(current);
    }

    pub fn sessions(&self) -> &Vec<Session> {
        &self.sessions
    }
}

pub trait Clock {
    fn now(&self) -> i64;
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Active {
        app_id: String,
        workspace: i32,
        // activity: ???,
    },
    Idle,
    Empty,
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
