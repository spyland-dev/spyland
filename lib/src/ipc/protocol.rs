/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Module that defines IPC protocol guidelines.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Result;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// A protocol version.
///
/// Used in [handshake](Request::Handshake) between backend and the daemon.
pub const VERSION: u32 = 0;

/// Request from the client.
///
/// Uses to request action for the server.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Request {
    /// A simple request that means checking the connection.
    Ping,

    /// A connection request from the backend to the daemon.
    Handshake {
        /// Backend protocol version.
        protocol_version: u32,
        /// Name of the backend.
        backend_name: String,
    },

    /// An event received from the backend.
    Event(spyland_core::Event),
}

/// Response from the server.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    /// A simple response that means connection is works.
    Pong,

    /// A response from the daemon to the backend.
    Handshake {
        /// Server protocol version.
        protocol_version: u32,
    },

    /// A [spyland_core::manager::SessionManager] response received as an answer to [Request::Event].
    EventResponse(spyland_core::manager::Response),
}

/// Low-level function, used to send `serializable` to `stream`.
///
/// # Arguments
/// * `stream` --- stream to send
/// * `serializable` --- data for send
pub fn send<T: Serialize>(stream: &UnixStream, serializable: T) -> Result<()> {
    let json = serde_json::to_string(&serializable)?;
    let mut writer = stream;
    writeln!(writer, "{json}")?;
    Ok(())
}

/// Low-level function, used to read `T` from the `stream`.
///
/// # Arguments
/// * `stream` --- stream to read
pub fn read<T: DeserializeOwned>(stream: &UnixStream) -> Result<T> {
    let mut json = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut json)?;

    let deserializable: T = serde_json::from_str(&json)?;

    Ok(deserializable)
}
