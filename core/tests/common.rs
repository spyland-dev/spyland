/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use spyland_core::Clock;
use spyland_core::Event;
use spyland_core::SessionManager;
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
        let mgr = SessionManager::new(clock.clone());

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

    pub fn event(&mut self, ev: Event) {
        self.mgr.handle_event(ev);
    }

    pub fn update_and_flush(&mut self) {
        self.mgr.update();
        self.mgr.flush();
    }
}
