/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use spyland_core::analytics::{SessionAnalytics, SessionGroup, group_sessions};
use spyland_core::{Event, Session, State};

mod common;
use common::TestDriver;

#[test]
fn analytic_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.advance(12);
    d.flush();

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(12);
    d.flush();

    d.event(Event::ActiveWindowChanged(None));
    d.advance(12);
    d.flush();

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.advance(12);
    d.flush();

    let a = SessionAnalytics::new(d.mgr.sessions().clone());

    assert_eq!(a.total_screen_time(), 36);
}

#[test]
fn analytic_app_time_test() {
    let mut d = TestDriver::new();

    const APP_ID: &str = "org.gnome.Loupe";

    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.advance(16);
    d.flush();

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(14);
    d.flush();

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.advance(20);
    d.flush();

    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.advance(4);
    d.flush();

    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.advance(6);
    d.flush();

    let a = SessionAnalytics::new(d.mgr.sessions().clone());

    assert_eq!(a.screen_time_app(APP_ID.into()), 20);
}

#[test]
fn analytic_idle_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some("kitty".into())));
    d.advance(40);

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(16);

    d.event(Event::ActiveWindowChanged(None));
    d.advance(24);

    d.event(Event::ActiveWindowChanged(Some("kitty".into())));
    d.advance(38);

    // d.event(Event::Idle(true));
    // d.advance(26);

    d.event(Event::ActiveWindowChanged(Some("kitty".into())));
    d.advance(2);

    let a = SessionAnalytics::new(d.mgr.sessions().clone());
    assert_eq!(a.idle_time(), 24);
}

#[test]
fn analytic_time_for_each_app_test() {
    let mut d = TestDriver::new();

    const APP_ID1: &str = "kitty";
    const APP_ID2: &str = "firefox";
    const APP_ID3: &str = "org.telegram.desktop";

    d.event(Event::ActiveWindowChanged(Some(APP_ID1.into())));
    d.advance(28);

    d.event(Event::ActiveWindowChanged(Some(APP_ID3.into())));
    d.advance(16);

    d.event(Event::ActiveWindowChanged(Some(APP_ID1.into())));
    d.advance(22);

    d.event(Event::ActiveWindowChanged(Some(APP_ID2.into())));
    d.advance(32);

    let a = SessionAnalytics::new(d.mgr.sessions().clone());
    let h = a.time_for_each_app();

    assert_eq!(h.len(), 3);

    let time = h[&State::Active {
        app_id: APP_ID1.into(),
        workspace: None,
    }];
    assert_eq!(time, 28 + 22);

    let time = h[&State::Active {
        app_id: APP_ID2.into(),
        workspace: None,
    }];
    assert_eq!(time, 32);

    let time = h[&State::Active {
        app_id: APP_ID3.into(),
        workspace: None,
    }];
    assert_eq!(time, 16);
}

#[test]
fn group_sessions_test() {
    const S1_APP_ID: &str = "firefox";
    const S1_START: i64 = 0;
    const S2_APP_ID: &str = "code";
    const S2_START: i64 = 10;

    let s1 = Session {
        start: S1_START,
        end: 10,
        state: State::Active {
            app_id: S1_APP_ID.into(),
            workspace: Some(1),
        },
    };
    let s2 = Session {
        start: S2_START,
        end: 20,
        state: State::Active {
            app_id: S2_APP_ID.into(),
            workspace: Some(2),
        },
    };
    let s3 = Session {
        start: 20,
        end: 30,
        state: State::Active {
            app_id: "steam".into(),
            workspace: Some(3),
        },
    };
    let s4 = Session {
        start: 30,
        end: 40,
        state: State::Idle,
    };

    let dev_group = SessionGroup {
        app_ids: vec![S2_APP_ID.into()],
        workspaces: vec![2],
    };
    let browser_group = SessionGroup {
        app_ids: vec![S1_APP_ID.into()],
        workspaces: vec![],
    };

    let groups = vec![dev_group.clone(), browser_group.clone()];
    let grouped = group_sessions(vec![s1, s2, s3, s4], &groups);

    {
        let group = grouped.get(&Some(dev_group.clone())).unwrap();
        assert_eq!(group.len(), 1);
        assert_eq!(group[0].start, S2_START);
    }

    {
        let group = grouped.get(&Some(browser_group.clone())).unwrap();
        assert_eq!(group.len(), 1);
        assert_eq!(group[0].start, S1_START);
    }

    assert_eq!(grouped.get(&None).unwrap().len(), 2);
}
