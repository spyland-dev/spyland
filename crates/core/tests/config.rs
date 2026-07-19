mod common;

use common::TestDriver;
use spyland_core::Event;
use spyland_core::manager::Configuration;

#[test]
fn flush_interval_test() {
    let mut d = TestDriver::new();
    const FLUSH_INTERVAL: i64 = 5;

    d.mgr.set_config(Configuration {
        flush_interval: FLUSH_INTERVAL,
        min_session_duration: 0,
        ..Default::default()
    });

    d.event(Event::ActiveWindowChanged(Some("alacritty".into())));
    d.tick(FLUSH_INTERVAL);

    assert_eq!(d.mgr.sessions().len(), 1);
}

#[test]
fn hidden_applications_test() {
    let mut d = TestDriver::new();

    const APP_ID: &str = "Terraria.bin.x86_64";

    d.mgr.set_config(Configuration {
        hidden_applications: vec![APP_ID.into()],
        ..Default::default()
    });

    d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
    d.tick(999999);

    d.flush();

    assert_eq!(d.mgr.sessions().len(), 0);
}

#[test]
fn min_session_duration_test() {
    let mut d = TestDriver::new();

    d.mgr.set_config(Configuration {
        min_session_duration: 10,
        ..Default::default()
    });

    d.event(Event::ActiveWindowChanged(Some("firefox".into())));
    d.advance(20);
    d.flush();
    assert_eq!(d.mgr.sessions().len(), 1);

    d.event(Event::ActiveWindowChanged(Some("zen".into())));
    d.advance(5);
    d.flush();
    assert_eq!(d.mgr.sessions().len(), 1);
}
