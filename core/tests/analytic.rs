/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use spyland_core::Event;
use spyland_core::SessionAnalytics;

mod common;
use common::TestDriver;

#[test]
fn analytic_test() {
    let mut d = TestDriver::new();

    d.event(Event::ActiveWindowChanged(Some("org.telegram.desktop".into())));
    d.advance(12);

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(12);

    d.event(Event::ActiveWindowChanged(Some("org.telegram.desktop".into())));
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

    d.event(Event::ActiveWindowChanged(Some("org.telegram.desktop".into())));
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
