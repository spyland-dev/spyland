/*
 *  spyland-backend-niri — niri Wayland compositor integration
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{
    collections::{HashMap, hash_map::Entry},
    path::PathBuf,
};

use anyhow::{Context, Result};
use log::warn;
use niri_ipc::{Event as NiriEvent, Request as NiriRequest, Window, socket::Socket};
use spyland_core::Event as CoreEvent;
use spyland_lib::{
    ipc::{
        IpcClient,
        protocol::{self, Request as IpcRequest},
    },
    path,
};

pub struct NiriBackend {
    socket_path: Option<PathBuf>,
    client: IpcClient,
}

impl NiriBackend {
    pub fn new(niri_socket_path: PathBuf, ipc_socket_path: PathBuf) -> Result<Self> {
        Ok(Self {
            socket_path: Some(niri_socket_path),
            client: IpcClient::new(ipc_socket_path)?,
        })
    }

    pub fn try_default() -> Result<Self> {
        Ok(Self {
            socket_path: None,
            client: IpcClient::new(path::get_socket_path()?)?,
        })
    }

    pub fn run(mut self) -> Result<()> {
        self.client
            .send_with_response(IpcRequest::Handshake {
                protocol_version: protocol::VERSION,
                backend_name: "niri".into(),
            })
            .context("Failed to handshake daemon")?;

        let mut event_socket = self
            .socket_path
            .as_ref()
            .map_or_else(Socket::connect, |p| Socket::connect_to(p))
            .context("Failed to connect to niri")?;

        event_socket
            .send(NiriRequest::EventStream)
            .context("Failed to send request to niri")?
            .ok()
            .context("Request return an error")?;

        let mut read_event = event_socket.read_events();
        let mut windows: HashMap<u64, Window> = HashMap::new();
        loop {
            match read_event() {
                Ok(event) => match event {
                    NiriEvent::WindowsChanged {
                        windows: niri_windows,
                    } => {
                        for window in niri_windows {
                            windows.insert(window.id, window.clone());
                            if window.is_focused {
                                self.client.send(IpcRequest::Event(
                                    CoreEvent::ActiveWindowChanged(window.app_id),
                                ))?;
                            }
                        }
                    }
                    NiriEvent::WindowOpenedOrChanged { window } => match windows.entry(window.id) {
                        Entry::Occupied(mut entry) => {
                            let entry = entry.get_mut();
                            *entry = window;
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(window);
                        }
                    },
                    NiriEvent::WindowClosed { id } => {
                        if let Entry::Occupied(entry) = windows.entry(id) {
                            entry.remove();
                        }
                    }
                    NiriEvent::WindowFocusChanged { id } => match id {
                        Some(id) => {
                            for w in windows.values_mut() {
                                if w.id == id {
                                    w.is_focused = true;
                                    self.client.send(IpcRequest::Event(
                                        CoreEvent::ActiveWindowChanged(w.app_id.clone()),
                                    ))?;
                                    break;
                                }
                            }
                        }
                        None => {
                            self.client
                                .send(IpcRequest::Event(CoreEvent::ActiveWindowChanged(None)))?;
                        }
                    },
                    NiriEvent::WorkspacesChanged { workspaces } => {
                        for workspace in workspaces {
                            if workspace.is_focused {
                                self.client.send(IpcRequest::Event(
                                    CoreEvent::WorkspaceChanged(workspace.id.try_into().unwrap()),
                                ))?;
                                break;
                            }
                        }
                    }
                    NiriEvent::WorkspaceActivated { id, focused } => {
                        if focused {
                            self.client
                                .send(IpcRequest::Event(CoreEvent::WorkspaceChanged(
                                    id.try_into().unwrap(),
                                )))?;
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    warn!("{:?}", e);
                    continue;
                }
            }
        }
    }
}
