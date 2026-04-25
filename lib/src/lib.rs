/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Public library API for spyland.
//!
//! There are three modules: [`db`], [`ipc`] and [`path`].
//! You can use [`db`] for accessing spyland database, and
//! [`ipc`] to communicate with spylandd (daemon).
//! [`path`] is a simple module to get spyland pathes.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod db;
pub mod ipc;
pub mod path;
