/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use spyland_lib::config::{ConfigFile, ConfigSection};
use std::fs;
use tempfile::Builder;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
struct TestCoreConfig {
    flush_interval: u64,
    min_session_duration: u64,
}

impl ConfigSection for TestCoreConfig {
    const SECTION: &'static str = "core";
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
struct TestBackendConfig {
    idle_timeout: u64,
    idle_on_overview: bool,
}

impl ConfigSection for TestBackendConfig {
    const SECTION: &'static str = "backend.niri";
}

#[test]
fn test_config_get_section() -> Result<()> {
    const CORE_FLUSH_INTERVAL: u64 = 30;
    const CORE_MIN_SESSION_DURATION: u64 = 10;
    const BACKEND_IDLE_TIMEOUT: u64 = 300;
    const BACKEND_IDLE_ON_OVERVIEW: bool = true;

    let temp_file = Builder::new().suffix(".toml").tempfile()?;
    let path = temp_file.path().to_path_buf();

    let config = format!(
        r#"
[core]
flush_interval = {CORE_FLUSH_INTERVAL}
min_session_duration = {CORE_MIN_SESSION_DURATION}

[backend.niri]
idle_timeout = {BACKEND_IDLE_TIMEOUT}
idle_on_overview = {BACKEND_IDLE_ON_OVERVIEW}
"#
    );
    fs::write(&path, config)?;

    let config_file = ConfigFile::new(path)?;

    let core: TestCoreConfig = config_file.get_section()?;
    assert_eq!(
        core,
        TestCoreConfig {
            flush_interval: CORE_FLUSH_INTERVAL,
            min_session_duration: CORE_MIN_SESSION_DURATION,
        }
    );

    let backend: TestBackendConfig = config_file.get_section()?;
    assert_eq!(
        backend,
        TestBackendConfig {
            idle_timeout: BACKEND_IDLE_TIMEOUT,
            idle_on_overview: BACKEND_IDLE_ON_OVERVIEW,
        }
    );

    Ok(())
}

#[test]
fn test_config_get_missing_section() -> Result<()> {
    let temp_file = Builder::new().suffix(".toml").tempfile()?;
    let path = temp_file.path().to_path_buf();

    fs::write(&path, "")?;

    let config_file = ConfigFile::new(path)?;

    let core: TestCoreConfig = config_file.get_section()?;
    assert_eq!(core, TestCoreConfig::default());

    Ok(())
}

#[test]
fn test_config_set_section() -> Result<()> {
    let temp_file = Builder::new().suffix(".toml").tempfile()?;
    let path = temp_file.path().to_path_buf();

    fs::write(&path, "[core]\nflush_interval = 15")?;

    let mut config_file = ConfigFile::new(path)?;

    const BACKEND_IDLE_TIMEOUT: u64 = 180;
    const BACKEND_IDLE_ON_OVERVIEW: bool = false;

    config_file
        .set_section(TestBackendConfig {
            idle_timeout: BACKEND_IDLE_TIMEOUT,
            idle_on_overview: BACKEND_IDLE_ON_OVERVIEW,
        })
        .unwrap();

    config_file.save()?;
    config_file.load()?;

    let backend: TestBackendConfig = config_file.get_section()?;
    assert_eq!(
        backend,
        TestBackendConfig {
            idle_timeout: BACKEND_IDLE_TIMEOUT,
            idle_on_overview: BACKEND_IDLE_ON_OVERVIEW,
        }
    );

    let core: TestCoreConfig = config_file.get_section()?;
    assert_eq!(core.flush_interval, 15);

    Ok(())
}

#[test]
fn test_config_by_name() -> Result<()> {
    let temp_file = Builder::new().suffix(".toml").tempfile()?;
    let path = temp_file.path().to_path_buf();

    const CORE_FLUSH_INTERVAL: u64 = 42;

    fs::write(
        &path,
        format!("[core]\nflush_interval = {CORE_FLUSH_INTERVAL}\n"),
    )?;

    let mut config_file = ConfigFile::new(path)?;

    let core: TestCoreConfig = config_file.get_section_by_name("core")?;
    assert_eq!(core.flush_interval, CORE_FLUSH_INTERVAL);

    const BACKEND_IDLE_TIMEOUT: u64 = 100;
    const BACKEND_IDLE_ON_OVERVIEW: bool = true;

    config_file.set_section_by_name(
        "backend.niri",
        TestBackendConfig {
            idle_timeout: 100,
            idle_on_overview: true,
        },
    )?;

    config_file.save()?;
    config_file.load()?;

    let backend: TestBackendConfig = config_file.get_section_by_name("backend.niri")?;
    assert_eq!(backend.idle_timeout, BACKEND_IDLE_TIMEOUT);
    assert_eq!(backend.idle_on_overview, BACKEND_IDLE_ON_OVERVIEW);

    Ok(())
}
