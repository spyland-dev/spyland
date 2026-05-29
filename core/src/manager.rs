/*
 *  spyland-core — session tracking engine
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! [SessionManager] and everything related to it here.

use crate::{Event, Session, State};

/// A trait that abstracts time for testability.
///
/// The [SessionManager] relies on this trait to get the current time,
/// which allows it to work with both real system time and mocked time in tests.
/// This makes the session tracking logic deterministic and testable.
///
/// # Example Implementation
///
/// For production code:
/// ```
/// # use spyland_core::manager::Clock;
/// struct SystemClock;
/// impl Clock for SystemClock {
///     fn now(&self) -> u64 {
///         std::time::SystemTime::now()
///             .duration_since(std::time::UNIX_EPOCH)
///             .unwrap()
///             .as_secs()
///     }
/// }
/// ```
///
/// For testing:
/// ```
/// # use spyland_core::manager::Clock;
/// struct MockClock {
///     current_time: u64,
/// }
/// impl Clock for MockClock {
///     fn now(&self) -> u64 {
///         self.current_time
///     }
/// }
/// ```
pub trait Clock {
    /// Returns the current time.
    ///
    /// Should return time as a UNIX timestamp (seconds since epoch),
    /// but it depends on the [Clock] implementation.
    fn now(&self) -> u64;
}

/// Structure that handles [Event]s and manages [Session]s.
pub struct SessionManager<C: Clock> {
    current: Session,
    workspace: Option<i32>,
    clock: C,
    sessions: Vec<Session>,
    old_session: Option<Session>,
    last_flush: u64,
    config: Configuration,
}

/// The [SessionManager] response.
///
/// It indicates what happens after [handling](SessionManager::handle_event) the [event](Event).
pub enum Response {
    /// The event was successfully processed with no additional information.
    Handled,
    /// The event was ignored, because something went wrong.
    ///
    /// For more information about the error, see documentation for the function.
    Ignored,

    /// The session was created.
    ///
    /// <div class="warning">
    ///
    /// If you want access to the session that was just created,
    /// don't forget to call [SessionManager::update] and [SessionManager::flush].
    ///
    /// </div>
    SessionCreated,
    /// The current session is hidden while idle.
    ///
    /// - `true` --- the current session was saved, and the new idle was created.
    /// - `false` --- the current idle session was ended, and the last session was restored.
    SessionIdled(bool),

    /// The current session saves to the internal session vector.
    /// For more information, see [SessionManager::flush].
    Flushed {
        /// Determines if the manager merged the last session and the current session.
        merged: bool,
    },
}

/// The [SessionManager] configuration. Use it to configure the manager's behavior.
///
/// Enable the `serde` feature to get the serde's serializing and deserializing.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Configuration {
    /// Interval of automatic flushes.
    ///
    /// See about [time abstraction](Clock) and [flushing](SessionManager::flush).
    pub flush_interval: u64,
    /// The list of hidden applications.
    ///
    /// [SessionManager] will ignore applications from this list.
    pub hidden_applications: Vec<String>,
    /// Minimal session duration.
    ///
    /// [Session]s that have less duration than this value will be filtered out.
    ///
    /// See [time abstraction](Clock).
    pub min_session_duration: u64,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            flush_interval: 15,
            hidden_applications: Vec::new(),
            min_session_duration: 5,
        }
    }
}

impl<C: Clock> SessionManager<C> {
    /// Creates a new instance of [SessionManager].
    ///
    /// # Arguments
    /// - `clock` --- a clock that the manager should use to get the current time.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            workspace: None,
            old_session: None,
            current: Session::new_empty(),
            sessions: Vec::new(),
            last_flush: 0,
            config: Configuration::default(),
        }
    }

    /// Handles the [Event].
    ///
    /// # Arguments
    /// - `event` --- an event to handle
    ///
    /// # Returns
    ///
    /// This function returns [Response], which represents the action that the manager did.
    /// The meaning of different responses varies depending on the event.
    ///
    /// - [Event::ActiveWindowChanged]
    ///     - [Response::SessionCreated]
    ///     - [Response::Ignored] --- the application of the session is hidden by [Configuration].
    /// - [Event::WorkspaceChanged]
    ///     - [Response::Handled] --- the workspace changed successfully (no other responses).
    /// - [Event::Idle]
    ///     - [Response::SessionIdled]
    ///     - [Response::Ignored] --- the session is already idle/non-idle.
    /// - [Event::Tick]
    ///     - [Response::Handled]
    pub fn handle_event(&mut self, event: Event) -> Response {
        match event {
            Event::ActiveWindowChanged(a) => {
                if let Some(ref app_id) = a {
                    if self.config.hidden_applications.contains(&app_id) {
                        return Response::Ignored;
                    }
                }

                self.new_session();

                self.current.state = match a {
                    Some(app_id) => State::Active {
                        app_id,
                        workspace: self.workspace,
                    },
                    None => State::Idle,
                };

                Response::SessionCreated
            }
            Event::WorkspaceChanged(id) => {
                self.workspace = Some(id);

                Response::Handled
            }
            Event::Idle(idle) => {
                if idle {
                    if self.current.state == State::Idle {
                        return Response::Ignored;
                    }

                    if !self.current.is_empty() {
                        self.update();
                        self.old_session = Some(self.current.clone());
                        self.flush();

                        self.current = Session::new_empty();
                    }

                    self.current.utc_start = self.clock.now();
                    self.current.state = State::Idle;
                    self.update();
                } else {
                    if self.current.state != State::Idle {
                        return Response::Ignored;
                    }

                    self.new_session();

                    self.current = self.old_session.clone().unwrap();
                    let now = self.clock.now();
                    self.current.utc_start = now;
                    self.current.utc_end = now;
                }

                Response::SessionIdled(idle)
            }
            Event::Tick => {
                let now = self.clock.now();

                self.update();

                if now - self.last_flush >= self.config.flush_interval {
                    self.last_flush = now;

                    return self.flush();
                }

                Response::Handled
            }
        }
    }

    fn new_session(&mut self) {
        let now = self.clock.now();

        self.current = Session::new_empty();
        self.current.utc_start = now;
    }

    /// Simply updates the end time of the current session.
    ///
    /// See [Self::flush].
    pub fn update(&mut self) {
        if self.current.is_empty() {
            return;
        }

        self.current.utc_end = self.clock.now();
    }

    /// Saves the current session to the internal sessions vector.
    ///
    /// This method persists the current session so it can be retrieved later via [Self::sessions].
    /// It also handles automatic merging of consecutive sessions with identical states.
    ///
    /// <div class="warning">
    ///
    /// Call [Self::update] before [Self::flush] to ensure the session's end time is current.
    ///
    /// </div>
    ///
    /// # Merging
    ///
    /// To prevent data fragmentation, consecutive sessions with the same [State] are automatically
    /// merged. For example, if the user stays in the same app for an hour but [Self::update]
    /// was called multiple times, only one merged session will be stored.
    ///
    /// # Duration Filter
    ///
    /// Sessions shorter than [Configuration::min_session_duration] are discarded (not stored).
    /// This prevents noise from brief focus changes. See [Configuration] for details.
    ///
    /// # Returns
    ///
    /// - [Response::Ignored] --- flush did not occur because:
    ///   - Current session is empty
    ///   - Session duration is less than [Configuration::min_session_duration]
    /// - [Response::Flushed] --- flush succeeded with the `merged` field indicating
    ///   whether this session was merged with the previous one
    pub fn flush(&mut self) -> Response {
        if self.current.is_empty() {
            return Response::Ignored;
        }

        let current = self.current.clone();

        if self.config.min_session_duration != 0 {
            if (current.utc_end - current.utc_start) <= self.config.min_session_duration {
                return Response::Ignored;
            }
        }

        if let Some(last) = self.sessions.last_mut() {
            if last.state == current.state {
                last.utc_end = current.utc_end;
                return Response::Flushed { merged: true };
            }
        }

        self.sessions.push(current);

        Response::Flushed { merged: false }
    }

    /// Returns the currently applied [Configuration].
    pub fn config(&self) -> &Configuration {
        &self.config
    }

    /// Sets the current [Configuration] of the manager with an argument `config`.
    pub fn set_config(&mut self, config: Configuration) {
        self.config = config;
    }

    /// Returns all saved sessions.
    ///
    /// This returns a reference to the internal vector of persisted sessions.
    /// To ensure all current activity is included, call [Self::update] and [Self::flush]
    /// before this method (unless an automatic flush has just occurred on [Event::Tick]).
    ///
    /// <div class="warning">
    ///
    /// The returned vector only includes sessions that have been flushed.
    /// The currently active session (stored in the manager's internal state)
    /// is not included until [Self::flush] is called.
    ///
    /// </div>
    ///
    /// # Example
    /// ```
    /// use spyland_core::manager::{SessionManager, Clock};
    ///
    /// struct MockClock { time: u64 }
    /// impl Clock for MockClock {
    ///     fn now(&self) -> u64 { self.time }
    /// }
    ///
    /// let mut manager = SessionManager::new(MockClock { time: 0 });
    /// manager.update();
    /// manager.flush();
    ///
    /// let sessions = manager.sessions();
    /// assert!(sessions.len() >= 0);
    /// ```
    pub fn sessions(&self) -> &Vec<Session> {
        &self.sessions
    }
}
