/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use spyland_core::Event;
use spyland_core::State;

mod common;
use common::TestDriver;

#[test]
fn simple_session() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.update_and_flush();

    assert_eq!(d.mgr.sessions().len(), 1, "Less then one sessions");
}

#[test]
fn session_time_test() {
    let mut d = TestDriver::new();

    const TIME: u64 = 30;

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

    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.tick(d.mgr.config().flush_interval);

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
            assert_eq!(
                WORKSPACE,
                workspace.expect("workspace is none"),
                "workspace not matching"
            );
        }
        _ => panic!("Incorrect state"),
    }
}

#[test]
fn simple_idle_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(None));
    d.update_and_flush();

    assert_eq!(d.mgr.sessions()[0].state, State::Idle);
}

#[test]
fn multiple_sessions_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(10);

    d.event(Event::ActiveWindowChanged(Some("kitty".into())));
    d.advance(10);

    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.advance(10);

    d.update_and_flush();

    let sessions = d.mgr.sessions();

    assert_eq!(sessions.len(), 3);
}

#[test]
fn direct_idle_test() {
    let mut d = TestDriver::new();

    d.event(Event::Idle(true));
    d.advance(5);

    d.update_and_flush();

    let sessions = d.mgr.sessions();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].state, State::Idle);
}

#[test]
fn unidle_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some(
        "Terraria.bin.x86_64".into(),
    )));
    d.advance(5);

    d.event(Event::Idle(true));
    d.advance(5);

    d.event(Event::Idle(false));
    d.advance(5);

    d.update_and_flush();

    let sessions = d.mgr.sessions();

    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].state, sessions[2].state);
    assert_eq!(sessions[2].utc_start, 11, "utc_start not matching");
    assert_eq!(sessions[2].utc_end, 16, "utc_end not matching");
}

#[test]
fn session_merge_test() {
    let mut d = TestDriver::new();
    let flush_interval = d.mgr.config().flush_interval;

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.tick(1);

    for i in 1..4 {
        d.advance(flush_interval);
        d.update_and_flush();

        let sessions = d.mgr.sessions();

        assert_eq!(sessions.len(), 1, "not one session");
        assert_eq!(sessions[0].utc_start, 1, "invalid utc_start");
        assert_eq!(
            sessions[0].utc_end,
            1 + flush_interval * i,
            "invalid utc_end"
        );
    }
}
