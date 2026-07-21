/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Provides basic tools like [grouping](group_sessions)
//! and [time calculation](SessionAnalytics) to analyze user [Session]s.

use crate::{Session, State};
use std::collections::HashMap;

/// A group of criteria to filter and classify sessions.
///
/// Holds application identifiers and workspace numbers to match against a [Session].
/// For grouping sessions see [`group_sessions()`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionGroup {
    /// List of application IDs. If empty, matches any application.
    pub app_ids: Vec<String>,
    /// List of workspace numbers. If empty, matches any workspace.
    pub workspaces: Vec<i32>,
}

impl SessionGroup {
    /// Checks if a session matches the group rules.
    pub fn matches(&self, session: &Session) -> bool {
        match &session.state {
            State::Active { app_id, workspace } => {
                let app_matches = self.app_ids.is_empty() || self.app_ids.contains(app_id);
                let ws_matches = self.workspaces.is_empty()
                    || workspace.is_some_and(|w| self.workspaces.contains(&w));

                app_matches && ws_matches
            }
            State::Idle => false,
        }
    }
}

/// Groups a list of sessions by the provided [SessionGroup] rules.
///
/// Returns a [HashMap] where keys are `Some(group)` for sessions matching a group,
/// or `None` for sessions that did not match any of the provided groups.
///
/// # Arguments
/// - `sessions` --- collection of sessions to group
/// - `groups` --- list of groups to match against
pub fn group_sessions(
    sessions: Vec<Session>,
    groups: &[SessionGroup],
) -> HashMap<Option<SessionGroup>, Vec<Session>> {
    let mut map: HashMap<Option<SessionGroup>, Vec<Session>> = HashMap::new();

    for session in sessions {
        let mut matched_group = None;
        for group in groups {
            if group.matches(&session) {
                matched_group = Some(group.clone());
                break;
            }
        }
        map.entry(matched_group).or_default().push(session);
    }

    map
}

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
    /// use spyland_core::analytics::SessionAnalytics;
    /// use spyland_core::{Session, State};
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
    /// assert_eq!(analytics.total_screen_time(), 30);
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
    /// use spyland_core::analytics::SessionAnalytics;
    /// use spyland_core::{Session, State};
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
    /// assert_eq!(analytics.total_screen_time(), 70);
    /// ```
    pub fn total_screen_time(&self) -> i64 {
        let mut counter: i64 = 0;

        for s in &self.sessions {
            if let State::Active { .. } = &s.state {
                counter += s.end - s.start;
            }
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
    /// use spyland_core::analytics::SessionAnalytics;
    /// # fn main() {
    /// # let sessions = Vec::new();
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// let screen_time = analytics.screen_time_app(String::from("org.telegram.desktop"));
    /// # }
    /// ```
    pub fn screen_time_app(&self, target_app_id: String) -> i64 {
        let mut counter: i64 = 0;

        for s in &self.sessions {
            if let State::Active { app_id, .. } = &s.state
                && *app_id == target_app_id
            {
                counter += s.end - s.start;
            }
        }

        counter
    }

    /// Returns total idle time.
    ///
    /// # Example
    /// ```
    /// use spyland_core::analytics::SessionAnalytics;
    /// # fn main() {
    /// # let sessions = Vec::new();
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// let total_idle_time = analytics.idle_time();
    /// # }
    /// ```
    pub fn idle_time(&self) -> i64 {
        let mut counter: i64 = 0;

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
    /// Returns [HashMap], where the key is a [State], and the value is a time ([u64]).
    ///
    /// # Example
    /// ```
    /// use spyland_core::{State, analytics::SessionAnalytics};
    /// # let sessions = Vec::new();
    /// let analytics = SessionAnalytics::new(sessions);
    ///
    /// let time_for_all_apps = analytics.time_for_each_app();
    ///
    /// for (state, time) in time_for_all_apps {
    ///     println!("{state}: {time} seconds");
    /// }
    /// ```
    pub fn time_for_each_app(&self) -> HashMap<State, i64> {
        let mut hash_map = HashMap::new();

        for s in &self.sessions {
            let duration = s.end - s.start;

            hash_map
                .entry(s.state.to_owned())
                .and_modify(|v| *v += duration)
                .or_insert(duration);
        }

        hash_map
    }
}
