/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

//! Module to communicate with spyland daemon.

use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

use anyhow::Result;

use self::protocol::{Request, Response};

pub mod protocol;

/// Simple IPC server.
/// [Accepts](`Self::accept`) [`IpcConnection`]s
/// Should only be using in the daemon.
pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    /// Creates new instance of [`IpcServer`]. Binds `path` as a socket.
    pub fn new(path: PathBuf) -> Result<Self> {
        let listener = UnixListener::bind(path)?;

        Ok(Self { listener })
    }

    /// Accepts a new connection to server socket.
    ///
    /// <div class="warning">That will block this thread until socket gets a client!</div>
    ///
    /// See [`UnixListener::accept`]
    pub fn accept(&mut self) -> Result<IpcConnection> {
        let (stream, _addr) = self.listener.accept()?;

        Ok(IpcConnection { stream })
    }
}

/// A connection to the [`IpcServer`].
pub struct IpcConnection {
    stream: UnixStream,
}

impl IpcConnection {
    /// Sends a [`Response`] to the stream.
    pub fn send(&self, response: Response) -> Result<()> {
        protocol::send(&self.stream, response)
    }

    /// Reads a [`Request`] to the stream.
    pub fn read(&self) -> Result<Request> {
        protocol::read(&self.stream)
    }

    /// Returns [`UnixStream`] of this client.
    pub fn stream(&self) -> &UnixStream {
        &self.stream
    }
}

/// Simple IPC client.
/// [Sends](IpcClient::send) [`Request`]s, [Reads](IpcClient::read) [`Response`]s.
pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    /// Creates new instance of [`IpcClient`]. Connects to socket by path `path`.
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            stream: UnixStream::connect(path)?,
        })
    }

    /// Returns [`UnixStream`] of this client.
    pub fn stream(&self) -> &UnixStream {
        &self.stream
    }

    /// Sends a [`Request`] to the stream.
    pub fn send(&mut self, request: Request) -> Result<()> {
        protocol::send(&self.stream, request)
    }

    /// Reads a [`Response`] to the stream.
    pub fn read(&mut self) -> Result<Response> {
        protocol::read(&self.stream)
    }

    /// Sends a [`Request`], and waiting for [`Response`].
    pub fn send_with_response(&mut self, request: Request) -> Result<Response> {
        self.send(request)?;
        self.read()
    }

    /// Tries to ping server.
    pub fn ping(&mut self) -> Result<bool> {
        let response = self.send_with_response(Request::Ping)?;

        Ok(response == Response::Pong)
    }
}
