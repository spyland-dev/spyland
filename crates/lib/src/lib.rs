/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Public library API for spyland.
//!
//! There are four modules to:
//! - [`config`] --- manage multiple config sections
//! - [`db`] --- access sessions database,
//! - [`ipc`] --- communicate with spyland daemon,
//! - [`path`] --- get spyland pathes.
//!
//! Every module has its feature. By default, they are all enabled.
//! If you don't need some module, then disable the corresponding feature.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "db")]
pub mod db;
#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(feature = "path")]
pub mod path;

#[cfg(not(any(feature = "db", feature = "ipc", feature = "path", feature = "config",)))]
compile_error!(
    "spyland-lib requires at least one feature to be enabled. Without any features,
     this is literally an empty crate. You may have done it by accident."
);
