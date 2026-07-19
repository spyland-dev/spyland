/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

#![allow(dead_code)]
#![allow(clippy::new_without_default)]

use spyland_core::Event;
use spyland_core::manager::{Clock, Configuration, Response, SessionManager};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct FakeClock {
    pub now: i64,
}

#[derive(Clone)]
pub struct SharedClock(pub Rc<RefCell<FakeClock>>);

impl Clock for SharedClock {
    fn now(&self) -> i64 {
        self.0.borrow().now
    }
}

pub struct TestDriver {
    pub mgr: SessionManager<SharedClock>,
    clock: SharedClock,
}

impl TestDriver {
    pub fn new() -> Self {
        let clock = SharedClock(Rc::new(RefCell::new(FakeClock { now: 1 })));
        let mut mgr = SessionManager::new(clock.clone());

        mgr.set_config(Configuration {
            min_session_duration: 0,
            ..Default::default()
        });

        Self { mgr, clock }
    }

    pub fn tick(&mut self, t: i64) {
        self.clock.0.borrow_mut().now = t;
        self.mgr.handle_event(Event::Tick);
    }

    pub fn advance(&mut self, t: i64) {
        self.clock.0.borrow_mut().now += t;
        self.mgr.handle_event(Event::Tick);
    }

    pub fn event(&mut self, ev: Event) -> Response {
        self.mgr.handle_event(ev)
    }

    pub fn flush(&mut self) {
        self.mgr.flush();
    }
}
