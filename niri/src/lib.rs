/*
 *  spyland-backend-niri — niri Wayland compositor integration
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */
use log::{error, warn};
use niri_ipc::socket::Socket;
use niri_ipc::{Event as NiriEvent, Request, Response, Window};
use spyland_core::{Backend, Event};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

pub struct NiriBackend {
    socket_path: Option<PathBuf>,
}

impl NiriBackend {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path: Some(socket_path),
        }
    }
}

impl Default for NiriBackend {
    fn default() -> Self {
        Self { socket_path: None }
    }
}

impl Backend for NiriBackend {
    fn is_available(&self) -> bool {
        match &self.socket_path {
            Some(p) => Socket::connect_to(p),
            None => Socket::connect(),
        }
        .is_ok()
    }

    fn subscribe(&mut self) -> mpsc::Receiver<spyland_core::Event> {
        let (tx, rx) = mpsc::channel();

        let path = self.socket_path.clone();
        thread::spawn(move || {
            run(tx, path);
        });

        rx
    }
}

macro_rules! send_event {
    ($tx:expr, $event:expr) => {
        if $tx.send($event).is_err() {
            error!("failed to send event");
            break;
        }
    };
}

fn run(tx: mpsc::Sender<Event>, socket_path: Option<PathBuf>) {
    let mut event_socket = socket_path
        .as_ref()
        .map_or_else(Socket::connect, |p| Socket::connect_to(p))
        .expect("failed to connect to niri");

    let reply = event_socket
        .send(Request::EventStream)
        .expect("failed to send request to niri");

    if matches!(reply, Ok(Response::Handled)) {
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
                                send_event!(tx, Event::ActiveWindowChanged(window.app_id));
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
                                    send_event!(tx, Event::ActiveWindowChanged(w.app_id.clone()));
                                    break;
                                }
                            }
                        }
                        None => {
                            send_event!(tx, Event::ActiveWindowChanged(None));
                        }
                    },
                    NiriEvent::WorkspacesChanged { workspaces } => {
                        for workspace in workspaces {
                            if workspace.is_focused {
                                send_event!(
                                    tx,
                                    Event::WorkspaceChanged(workspace.id.try_into().unwrap())
                                );
                                break;
                            }
                        }
                    }
                    NiriEvent::WorkspaceActivated { id, focused } => {
                        if focused {
                            send_event!(tx, Event::WorkspaceChanged(id.try_into().unwrap()))
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
