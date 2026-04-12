/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use log::{error, warn};
use niri_ipc::socket::Socket;
use niri_ipc::{Event as NiriEvent, Request, Response, Window};
use spyland_core::{Backend, Event};
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

fn resolve_window(socket: &mut Socket, id: u64) -> Option<Window> {
    let response = socket.send(Request::Windows).ok()?.ok()?;

    if let Response::Windows(windows) = response {
        return windows.iter().find(|w| w.id == id).cloned();
    }

    None
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
    let connect = || {
        socket_path
            .as_ref()
            .map_or_else(Socket::connect, |p| Socket::connect_to(p))
            .expect("failed to connect to niri")
    };
    let mut event_socket = connect();
    let mut query_socket = connect();

    let reply = event_socket
        .send(Request::EventStream)
        .expect("failed to send request to niri");

    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = event_socket.read_events();
        loop {
            match read_event() {
                Ok(event) => match event {
                    NiriEvent::WindowFocusChanged { id } => match id {
                        Some(id) => {
                            let window = resolve_window(&mut query_socket, id);
                            if let Some(w) = window {
                                send_event!(tx, Event::ActiveWindowChanged(w.app_id));
                            }
                        }
                        None => {
                            send_event!(tx, Event::ActiveWindowChanged(None));
                        }
                    },
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
