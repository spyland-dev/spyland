/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::{Session, State};
use std::collections::HashMap;

/// Statistics for sessions.
///
/// Provides methods to calculate various metrics from a collection of sessions,
/// such as total screen time, per-application usage, and idle time.
pub struct SessionAnalytics {
    sessions: Vec<Session>,
}

impl SessionAnalytics {
    /// Creates a new instance from a list of sessions.
    ///
    /// # Arguments
    /// - `sessions` --- collected sessions to analyze
    ///
    /// # Example
    /// ```
    /// use spyland_core::{SessionAnalytics, Session, State};
    ///
    /// let sessions = vec![
    ///     Session {
    ///         start: 100,
    ///         end: 130,
    ///         state: State::Active {
    ///             app_id: "firefox".to_string(),
    ///             workspace: Some(1),
    ///         },
    ///     },
    ///     Session {
    ///         start: 130,
    ///         end: 150,
    ///         state: State::Idle,
    ///     },
    /// ];
    ///
    /// let analytics = SessionAnalytics::new(sessions);
    /// assert_eq!(analytics.total_screen_time(), 50);
    /// ```
    pub fn new(sessions: Vec<Session>) -> Self {
        Self { sessions }
    }

    /// Returns the total screen time across all sessions.
    ///
    /// Sums the duration of all sessions regardless of application or state.
    /// Time is measured in the same units as returned by [Clock::now].
    ///
    /// # Example
    /// ```
    /// use spyland_core::{SessionAnalytics, Session, State};
    ///
    /// let sessions = vec![
    ///     Session {
    ///         start: 0,
    ///         end: 50,
    ///         state: State::Idle
    ///     },
    ///     Session {
    ///         start: 50,
    ///         end: 120,
    ///         state: State::Active {
    ///             app_id: "app".to_string(),
    ///             workspace: None
    ///         }
    ///     },
    /// ];
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// assert_eq!(analytics.total_screen_time(), 120);
    /// ```
    pub fn total_screen_time(&self) -> u64 {
        let mut counter: u64 = 0;

        for s in &self.sessions {
            counter += s.end - s.start;
        }

        counter
    }

    /// Returns screen time for a specific app.
    ///
    /// # Arguments
    /// - `target_app_id` --- target application identifier
    ///
    /// # Example
    /// ```
    /// use spyland_core::SessionAnalytics;
    /// # fn main() {
    /// # let sessions = Vec::new();
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// let screen_time = analytics.screen_time_app(String::from("org.telegram.desktop"));
    /// # }
    /// ```
    pub fn screen_time_app(&self, target_app_id: String) -> u64 {
        let mut counter: u64 = 0;

        for s in &self.sessions {
            if let State::Active { app_id, .. } = &s.state {
                if *app_id == target_app_id {
                    counter += s.end - s.start;
                }
            }
        }

        counter
    }

    /// Returns total idle time.
    ///
    /// # Example
    /// ```
    /// use spyland_core::SessionAnalytics;
    /// # fn main() {
    /// # let sessions = Vec::new();
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// let total_idle_time = analytics.idle_time();
    /// # }
    /// ```
    pub fn idle_time(&self) -> u64 {
        let mut counter: u64 = 0;

        for s in &self.sessions {
            if let State::Idle = &s.state {
                counter += s.end - s.start;
            }
        }

        counter
    }

    /// Returns time for all applications.
    ///
    /// # Returns
    /// Returns [HashMap], where the key is an application identifier ([String]), and the value is a time ([u64]).
    ///
    /// # Example
    /// ```
    /// use spyland_core::SessionAnalytics;
    /// # let sessions = Vec::new();
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// let time_for_all_apps = analytics.time_for_each_app();
    ///
    /// for (key, value) in time_for_all_apps {
    ///     println!("Application: {key}, Time: {value} seconds");
    /// }
    /// ```
    pub fn time_for_each_app(&self) -> HashMap<String, u64> {
        let mut hash_map = HashMap::new();

        for s in &self.sessions {
            if let State::Active { app_id, .. } = &s.state {
                let duration = s.end - s.start;

                hash_map
                    .entry(app_id.to_owned())
                    .and_modify(|v| *v += duration)
                    .or_insert(duration);
            }
        }

        hash_map
    }
}
