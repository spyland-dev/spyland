//! A module to get spyland [database](get_database_path) and [socket](get_socket_path) paths.
//! There are also safe versions that verify that the path is usable.
//!
//! <div class="warning">
//! It is important to know that debug builds use a different filenames
//! to avoid using a release build and a developer build at the same time.
//! </div>

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
///
/// <div class="warning">
/// Don't forget that debug builds use a different name: `sessions-debug.sqlite`!
/// </div>
pub fn get_database_path() -> Result<PathBuf> {
    let state_path = match env::var("XDG_STATE_HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            let home = env::var("HOME").context("Home directory is not set")?;
            PathBuf::from(home).join(".local/state/")
        }
    }
    .join("spyland");

    let filename = if cfg!(debug_assertions) {
        "sessions-debug.sqlite"
    } else {
        "sessions.sqlite"
    };

    Ok(state_path.join(filename))
}

define_path_ensurer!(
    /// Returns and ensures that the path exists.
    ///
    /// <div class="warning">
    /// But it doesn't make sure the file exists, because sqlite will automatically create the file if it needs to.
    ///
    /// Don't forget that debug builds use a different name: `sessions-debug.sqlite`!
    /// </div>
    ensure_database_path,
    get_database_path
);

/// Returns the path to the spyland socket.
///
/// `$XDG_RUNTIME_DIR/spyland.sock` or
/// `$XDG_RUNTIME_DIR/spyland-debug.sock` **(IN DEBUG BUILDS!)**
pub fn get_socket_path() -> Result<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")?;

    let filename = if cfg!(debug_assertions) {
        "spyland-debug.sock"
    } else {
        "spyland.sock"
    };

    Ok(PathBuf::from(runtime_dir).join(filename))
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
///
/// <div class="warning">
/// If the socket already exists, it will be removed!
/// Use carefully so as not to interfere with the running daemon.
/// </div>
pub fn ensure_socket_path() -> Result<PathBuf> {
    let sock_path = get_socket_path()?;

    let parent = &sock_path.parent().context("Path parent was None")?;
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    if sock_path.exists() {
        fs::remove_file(&sock_path)?;
    }

    Ok(sock_path)
}
