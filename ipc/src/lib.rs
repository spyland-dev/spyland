/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

use anyhow::Result;

use crate::protocol::{Request, Response};

pub mod protocol;

pub struct IpcServer {
    stream: UnixStream,
}

impl IpcServer {
    pub fn new(path: PathBuf) -> Result<Self> {
        let listener = UnixListener::bind(path)?;
        let (stream, _addr) = listener.accept()?;

        Ok(Self { stream: stream })
    }

    pub fn stream(&self) -> &UnixStream {
        &self.stream
    }

    pub fn send(&self, response: Response) -> Result<()> {
        protocol::send(&self.stream, response)
    }

    pub fn read(&self) -> Result<Request> {
        protocol::read(&self.stream)
    }
}

pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            stream: UnixStream::connect(path)?,
        })
    }

    pub fn stream(&self) -> &UnixStream {
        &self.stream
    }

    pub fn send(&mut self, request: Request) -> Result<()> {
        protocol::send(&self.stream, request)
    }

    pub fn read(&mut self) -> Result<Response> {
        protocol::read(&self.stream)
    }

    pub fn send_with_response(&mut self, request: Request) -> Result<Response> {
        self.send(request)?;
        self.read()
    }

    pub fn ping(&mut self) -> Result<bool> {
        let response = self.send_with_response(Request::Ping)?;

        Ok(response == Response::Pong)
    }
}
