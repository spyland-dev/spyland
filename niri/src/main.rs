/*
 *  spyland-backend-niri — niri Wayland compositor integration
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use log::info;

mod backend;

fn main() {
    env_logger::init();

    info!("Starting niri backend...");
}
