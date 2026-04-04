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
    workspace: Option<i32>,
    clock: C,
    sessions: Vec<Session>,
    last_flush: i64,
}

pub const SESSION_MANAGER_FLUSH_INTERVAL: i64 = 15;

impl<C: Clock> SessionManager<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            workspace: None,
            current: Session::new_empty(),
            sessions: Vec::new(),
            last_flush: 0,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::ActiveWindowChanged(a) => {
                self.new_session();

                self.current.state = match a {
                    Some(app_id) => {
                        State::Active {
                            app_id,
                            workspace: self.workspace,
                        }
                    }
                    None => State::Idle
        }
            }
            Event::WorkspaceChanged(id) => {
                self.workspace = Some(id);
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
            self.update();
            self.flush();
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

pub struct SessionAnalytics {
    sessions: Vec<Session>,
}

impl SessionAnalytics {
    pub fn new(sessions: Vec<Session>) -> Self {
        Self {
            sessions,
        }
    }

    pub fn total_screen_time(&self) -> i64 {
        let mut counter: i64 = 0;

        for s in &self.sessions {
            counter += s.utc_end - s.utc_start;
        }

        counter
    }

    pub fn screen_time_app(&self, target_app_id: String) -> i64 {
        let mut counter: i64 = 0;

        for s in &self.sessions {
            if let State::Active { app_id, .. } = &s.state  {
                if *app_id == target_app_id {
                    counter += s.utc_end - s.utc_start;
                }
            }
        }

        counter
    }

    pub fn idle_time(&self) -> i64 {
        let mut counter: i64 = 0;

        for s in &self.sessions {
            if let State::Idle = &s.state {
                counter += s.utc_end - s.utc_start;
            }
        }

        counter
    }
}

pub trait Clock {
    fn now(&self) -> i64;
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

pub enum Event {
    ActiveWindowChanged(Option<String>),
    WorkspaceChanged(i32),
    Idle(bool),
    Tick,
}

pub trait Backend {
    fn is_available(&self) -> bool;
    fn subscribe(&mut self) -> Receiver<Event>;
}
