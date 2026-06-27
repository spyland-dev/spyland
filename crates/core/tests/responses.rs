/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

mod common;
use common::*;
use spyland_core::{
    Event, State,
    manager::{Configuration, Response},
};

#[test]
fn event_active_window_responses() {
    let mut d = TestDriver::new();

    let r = d.event(Event::ActiveWindowChanged(Some("kitty".to_string())));
    assert_eq!(r, Response::SessionCreated, "Response was not Handled");
    d.advance(5);
    d.flush();
    assert_eq!(
        d.mgr.sessions().len(),
        1,
        "Response received invalid information: Expected session to be saved"
    );

    const HIDDEN_APP_ID: &str = "firefox";

    d.mgr.set_config(Configuration {
        hidden_applications: vec![HIDDEN_APP_ID.into()],
        ..Default::default()
    });

    let r = d.event(Event::ActiveWindowChanged(Some(HIDDEN_APP_ID.into())));
    assert_eq!(r, Response::Ignored, "Expected response to be Ignored");
    d.advance(5);
    d.flush();
    assert_eq!(
        d.mgr.sessions().len(),
        1,
        "Response received invalid information: Expected session to be ignored"
    );
}

#[test]
fn event_workspace_response() {
    let mut d = TestDriver::new();

    const WORKSPACE: i32 = 1;

    let r = d.event(Event::WorkspaceChanged(WORKSPACE));
    assert_eq!(r, Response::Handled, "Response was not Handled");

    const APP_ID: &str = "discord";

    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.advance(5);
    d.flush();

    assert_eq!(d.mgr.sessions().len(), 1, "Failed to save session");
    match &d.mgr.sessions()[0].state {
        State::Active { app_id, workspace } => {
            assert_eq!(app_id, APP_ID);
            assert_eq!(workspace.expect("Workspace was None"), WORKSPACE);
        }
        _ => panic!("Unexpected state"),
    };
}

#[test]
fn event_idle_responses() {
    let mut d = TestDriver::new();

    let r = d.event(Event::Idle(true));
    assert_eq!(r, Response::Handled, "Expected Handled response");
    d.advance(5);
    d.flush();

    assert_eq!(d.mgr.sessions().len(), 1, "Session was not created");
    assert_eq!(
        d.mgr.sessions()[0].state,
        State::Idle,
        "Session is not idle"
    );

    let r = d.event(Event::Idle(true));
    assert_eq!(r, Response::Ignored, "Expected response to be Ignored");

    let r = d.event(Event::Idle(false));
    assert_eq!(r, Response::Handled, "Expected Handled response");
}

#[test]
fn event_idle_sessions_responses() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some("steam".into())));
    d.advance(5);
    d.flush();

    let r = d.event(Event::Idle(true));
    assert_eq!(
        r,
        Response::SessionIdled(true),
        "Expected response to be Idled"
    );

    let r = d.event(Event::Idle(false));
    assert_eq!(
        r,
        Response::SessionIdled(false),
        "Expected response to be Unidled"
    );
}

#[test]
fn event_tick_responses() {
    let mut d = TestDriver::new();

    let r = d.event(Event::Tick);
    assert_eq!(r, Response::Ignored, "Expected response to be Ignored");

    d.event(Event::ActiveWindowChanged(Some("kitty".into())));
    let r = d.event(Event::Tick);
    assert_eq!(r, Response::Handled, "Expected response to be Handled");
}
