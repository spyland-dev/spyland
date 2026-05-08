/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{collections::HashMap, sync::mpsc::Receiver};

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

pub struct SessionManager<C: Clock> {
    current: Session,
    workspace: Option<i32>,
    clock: C,
    sessions: Vec<Session>,
    old_session: Option<Session>,
    last_flush: u64,
    config: Configuration,
}

pub enum Response {
    Handled,
    Ignored,

    SessionCreated,
    SessionUpdated,
    SessionIdled(bool),

    Flushed{ merged: bool },
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Configuration {
    pub flush_interval: u64,
    pub hidden_applications: Vec<String>,
    pub min_session_duration: Option<u64>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            flush_interval: 15,
            hidden_applications: Vec::new(),
            min_session_duration: Some(5),
        }
    }
}

impl<C: Clock> SessionManager<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            workspace: None,
            old_session: None,
            current: Session::new_empty(),
            sessions: Vec::new(),
            last_flush: 0,
            config: Configuration::default(),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Response {
        match event {
            Event::ActiveWindowChanged(a) => {
                if let Some(ref app_id) = a {
                    if self.config.hidden_applications.contains(&app_id) {
                        return Response::Ignored;
                    }
                }

                self.new_session();

                self.current.state = match a {
                    Some(app_id) => State::Active {
                        app_id,
                        workspace: self.workspace,
                    },
                    None => State::Idle,
                };

                Response::SessionCreated
            }
            Event::WorkspaceChanged(id) => {
                self.workspace = Some(id);

                Response::Handled
            }
            Event::Idle(idle) => {
                if idle {
                    if self.current.state == State::Idle {
                        return Response::Ignored;
                    }

                    if !self.current.is_empty() {
                        self.update();
                        self.old_session = Some(self.current.clone());
                        self.flush();

                        self.current = Session::new_empty();
                    }

                    self.current.utc_start = self.clock.now();
                    self.current.state = State::Idle;
                    self.update();
                } else {
                    if self.current.state != State::Idle {
                        return Response::Ignored;
                    }

                    self.new_session();

                    self.current = self.old_session.clone().unwrap();
                    let now = self.clock.now();
                    self.current.utc_start = now;
                    self.current.utc_end = now;
                }

                Response::SessionIdled(idle)
            }
            Event::Tick => {
                let now = self.clock.now();

                self.update();

                if now - self.last_flush >= self.config.flush_interval {
                    self.last_flush = now;

                    return self.flush();
                }

                Response::Handled
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

    pub fn flush(&mut self) -> Response {
        if self.current.is_empty() {
            return Response::Ignored;
        }

        let current = self.current.clone();

        if let Some(min) = self.config.min_session_duration {
            if (current.utc_end - current.utc_start) <= min {
                return Response::Ignored;
            }
        }

        if let Some(last) = self.sessions.last_mut() {
            if last.state == current.state {
                last.utc_end = current.utc_end;
                return Response::Flushed { merged: true };
            }
        }

        self.sessions.push(current);

        Response::Flushed { merged: false}
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn set_config(&mut self, config: Configuration) {
        self.config = config;
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

pub trait Clock {
    fn now(&self) -> u64;
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

pub trait Backend {
    fn is_available(&self) -> bool;
    fn subscribe(&mut self) -> Receiver<Event>;
}
