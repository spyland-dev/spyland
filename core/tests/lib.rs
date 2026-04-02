/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use spyland_core::Clock;
use spyland_core::Event;
use spyland_core::SESSION_MANAGER_FLUSH_INTERVAL as FLUSH_INTERVAL;
use spyland_core::SessionManager;
use spyland_core::State;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
struct FakeClock {
    pub now: i64,
}

#[derive(Clone)]
struct SharedClock(pub Rc<RefCell<FakeClock>>);

impl Clock for SharedClock {
    fn now(&self) -> i64 {
        self.0.borrow().now
    }
}

struct TestDriver {
    mgr: SessionManager<SharedClock>,
    clock: SharedClock,
}

impl TestDriver {
    fn new() -> Self {
        let clock = SharedClock(Rc::new(RefCell::new(FakeClock { now: 1 })));
        let mgr = SessionManager::new(clock.clone());

        Self { mgr, clock }
    }

    fn tick(&mut self, t: i64) {
        self.clock.0.borrow_mut().now = t;
        self.mgr.handle_event(Event::Tick);
    }

    fn advance(&mut self, t: i64) {
        self.clock.0.borrow_mut().now += t;
        self.mgr.handle_event(Event::Tick);
    }

    fn event(&mut self, ev: Event) {
        self.mgr.handle_event(ev);
    }

    fn update_and_flush(&mut self) {
        self.mgr.update();
        self.mgr.flush();
    }
}

#[test]
fn simple_session() {
    let mut d = TestDriver::new();

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.update_and_flush();

    assert_eq!(d.mgr.sessions().len(), 1, "Less then one sessions");
}

#[test]
fn time_test() {
    let mut d = TestDriver::new();

    const TIME: i64 = 30;

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.tick(TIME);
    // d.update_and_flush();
    // not needed because of automatic update()
    // and refresh() in SessionManager

    assert_eq!(d.mgr.sessions()[0].utc_start, 1, "Incorrect start time");
    assert_eq!(d.mgr.sessions()[0].utc_end, TIME, "Incorrect end time");
}

#[test]
fn auto_flush_test() {
    let mut d = TestDriver::new();

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.tick(FLUSH_INTERVAL);

    assert_eq!(d.mgr.sessions().len(), 1, "Not one session");
}

#[test]
fn session_data_test() {
    let mut d = TestDriver::new();

    const WORKSPACE: i32 = 1;
    const APP_ID: &str = "firefox";

    d.event(Event::WorkspaceChanged(WORKSPACE));
    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.update_and_flush();

    match &d.mgr.sessions()[0].state {
        State::Active { app_id, workspace } => {
            assert_eq!(APP_ID, app_id, "app_id not matching");
            assert_eq!(WORKSPACE, *workspace, "workspace not matching");
        }
        _ => panic!("Incorrect state"),
    }
}

#[test]
fn simple_idle_test() {
    let mut d = TestDriver::new();

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(None));
    d.update_and_flush();

    assert_eq!(d.mgr.sessions()[0].state, State::Idle);
}

#[test]
fn multiple_sessions_test() {
    let mut d = TestDriver::new();

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(10);

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("kitty".into())));
    d.advance(10);

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.advance(10);

    d.update_and_flush();

    let sessions = d.mgr.sessions();

    assert_eq!(sessions.len(), 3);
}
