//! A module to get spyland [database](get_database_path) and [socket](get_socket_path) paths.
//! There are also safe versions that verify that the path is usable.

use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

macro_rules! define_path_ensurer {
    (
        $(#[$meta:meta])*
        $fn_name:ident,
        $get_path_fn:ident
        $(, |$__path:ident| $extra:block)?
    ) => {
        $(#[$meta])*
        pub fn $fn_name() -> anyhow::Result<std::path::PathBuf> {
            use anyhow::Context;
            use std::fs;

            let path = $get_path_fn()?;

            let parent = path
                .parent()
                .context("Path parent was None")?;

            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }

            $(
                {
                    let $__path = &path;
                    $extra
                }
            )?

            Ok(path)
        }
    };
}

/// Returns the path to the spyland database:
/// `$XDG_STATE_HOME/spyland/sessions.sqlite` or
/// `$HOME/.local/state/spyland/sessions.sqlite` (fallback).
pub fn get_database_path() -> Result<PathBuf> {
    let state_path = match env::var("XDG_STATE_HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            let home = env::var("HOME").context("Home directory is not set")?;
            PathBuf::from(home).join(".local/state/")
        }
    }
    .join("spyland/sessions.sqlite");

    Ok(state_path)
}

define_path_ensurer!(
    /// Returns and ensures that the path exists.
    ///
    /// <div class="warning">
    /// But it doesn't make sure the file exists, because sqlite will automatically create the file if it needs to.
    /// </div>
    ensure_database_path,
    get_database_path
);

/// Returns the path to the spyland socket.
///
/// `$XDG_RUNTIME_DIR/spyland.sock`
pub fn get_socket_path() -> Result<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")?;

    Ok(PathBuf::from(runtime_dir).join("spyland.sock"))
}

define_path_ensurer!(
    /// Returns the socket path and ensures that it is not already occupied.
    ///
    /// <div class="warning">
    /// If the socket already exists, it will be removed!
    /// Use carefully so as not to interfere with the running daemon.
    /// </div>
    ensure_socket_path,
    get_socket_path,
    |__path| {
        if __path.exists() {
            fs::remove_file(__path)?;
        }
    }
);

/// Returns the configuration path.
///
/// `$XDG_CONFIG_HOME/spyland/config.toml`
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = match env::var("XDG_CONFIG_HOME") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            let home = env::var("HOME").context("Home directory is not set")?;
            PathBuf::from(home).join(".config/")
        }
    }.join("spyland/config.toml");

    Ok(config_dir)
}

define_path_ensurer!(
    /// Returns and ensures that the configuration file exists.
    ensure_config_path,
    get_config_path,
    |__path| {
        if !__path.exists() {
            fs::File::create(__path)?;
        }
    }
);
