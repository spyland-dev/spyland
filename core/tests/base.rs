/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use spyland_core::Clock;
use spyland_core::Event;
use spyland_core::SessionManager;
use spyland_core::State;

#[derive(Clone, Copy)]
pub struct FakeClock {
    pub now: i64,
}

impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.now
    }
}

impl FakeClock {
    pub fn set(&mut self, t: i64) {
        self.now = t;
    }
}

struct TestDriver {
    mgr: SessionManager<FakeClock>,
    clock: FakeClock,
}

impl TestDriver {
    fn new() -> Self {
        let clock = FakeClock { now: 0 };
        let mgr = SessionManager::new(clock);

        Self { mgr, clock }
    }

    fn tick(&mut self, t: i64) {
        self.clock.set(t);
        self.mgr.handle_event(Event::Tick);
    }

    fn event(&mut self, ev: Event) {
        self.mgr.handle_event(ev);
    }
}

#[test]
fn simple_session() {
    let mut d = TestDriver::new();

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.tick(1);

    assert_eq!(d.mgr.sessions().len(), 1);
}

#[test]
fn end_time_test() {
    let mut d = TestDriver::new();

    const TIME: i64 = 30;

    d.event(Event::WorkspaceChanged(0));
    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.tick(TIME);

    assert_eq!(d.mgr.sessions()[0].utc_end, TIME);
}

#[test]
fn session_data_test() {
    let mut d = TestDriver::new();

    const WORKSPACE: i32 = 1;
    const APP_ID: &str = "firefox";

    d.event(Event::WorkspaceChanged(WORKSPACE));
    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.tick(1);

    match &d.mgr.sessions()[0].state {
        State::Active { app_id, workspace } => {
            assert_eq!(APP_ID, app_id);
            assert_eq!(WORKSPACE, *workspace);
        }
        _ => panic!("Invalid state"),
    }
}
