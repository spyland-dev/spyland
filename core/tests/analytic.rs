/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use spyland_core::Event;
use spyland_core::SessionAnalytics;

mod common;
use common::TestDriver;

#[test]
fn analytic_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.advance(12);

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(12);

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.advance(12);

    d.update_and_flush();

    let a = SessionAnalytics::new(d.mgr.sessions().clone());

    assert_eq!(a.total_screen_time(), 36);
}

#[test]
fn analytic_app_time_test() {
    let mut d = TestDriver::new();

    const APP_ID: &str = "org.gnome.Loupe";

    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.advance(16);

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(14);

    d.event(Event::ActiveWindowChanged(Some(
        "org.telegram.desktop".into(),
    )));
    d.advance(20);

    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.advance(4);

    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.advance(6);

    d.update_and_flush();

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

    let time = h[APP_ID1];
    assert_eq!(time, 28 + 22);

    let time = h[APP_ID2];
    assert_eq!(time, 32);

    let time = h[APP_ID3];
    assert_eq!(time, 16);
}
