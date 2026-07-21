/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Session tracking engine.
//!
//! Abstraction for tracking screen time, [events](Event), [sessions](Session),
//! and [user states](State).

#![deny(missing_docs)]

pub mod analytics;

pub mod manager;

/// Abstraction of user sessions.
///
/// A simple data structure representing a continuous period of user activity.
/// It contains start and end times and the user's state during this period.
#[derive(Clone)]
pub struct Session {
    /// Session start time (UNIX timestamp in seconds).
    ///
    /// The exact format depends on the [Clock](manager::Clock) implementation.
    pub start: i64,

    /// Session end time (UNIX timestamp in seconds).
    ///
    /// The exact format depends on the [Clock](manager::Clock) implementation.
    pub end: i64,

    /// User state during this session.
    pub state: State,
}

/// User state abstraction.
///
/// Represents what the user is currently doing or their activity status.
/// Used within [Session] to describe the context of screen time.
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    /// Active state: user is focused on an application window.
    ///
    /// The application ID indicates which app is in focus.
    Active {
        /// The ID of the application that is currently focused.
        app_id: String,
        /// Workspace number where the window is focused (compositor-specific).
        ///
        /// May be `None` if the compositor does not support workspaces.
        workspace: Option<i32>,
        // TODO: ACTIVITIES
    },
    /// Idle state: user is not interacting with any application.
    ///
    /// This typically occurs when the screensaver activates or a window loses focus
    /// without another window gaining it.
    Idle,
}

/// An abstraction of events from a Wayland compositor.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq)]
pub enum Event {
    /// The focus of the window changed.
    ///
    /// - `Some(app_id)` --- focus changed to another window
    /// - `None` --- focus changed to nothing
    ActiveWindowChanged(Option<String>),
    /// The current workspace changed.
    ///
    /// The value is a new workspace ID.
    WorkspaceChanged(i32),
    /// The idle state has changed.
    ///
    /// - `true` --- set the current state to [State::Idle]
    /// - `false` --- remove that state and return to the previous
    Idle(bool),
    /// The current time has changed.
    ///
    /// That event causes an automatic flush. For more information,
    /// see [flushing](manager::SessionManager::flush).
    Tick,
}
