/*
 *  spyland-backend-niri — niri Wayland compositor integration
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

mod common;
use common::*;

#[test]
fn window_test() {
    let mut d = TestDriver::new();

    let (_, app_id) = d.new_test_window();

    d.assert_event(CoreEvent::ActiveWindowChanged(Some(app_id)));
}

#[test]
fn workspace_test() {
    let d = TestDriver::new();

    const WORKSPACE_ID: u64 = 0;

    d.send(NiriEvent::WorkspaceActivated {
        id: WORKSPACE_ID,
        focused: true,
    });

    d.assert_event(CoreEvent::WorkspaceChanged(
        WORKSPACE_ID.try_into().unwrap(),
    ));
}
