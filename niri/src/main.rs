/*
 *  spyland-backend-niri — niri Wayland compositor integration
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::{Context, Result};
use log::info;
use spyland_backend_niri::NiriBackend;

fn main() -> Result<()> {
    env_logger::init();

    info!("Starting niri backend...");

    let backend = NiriBackend::try_default().context("Failed to start backend")?;

    backend.run()?;

    Ok(())
}
