/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use niri_ipc::socket::Socket;
use niri_ipc::{Event as NiriEvent, Request, Response, Window};
use spyland_core::{Backend, Event};
use std::sync::mpsc;
use std::thread;

pub struct NiriBackend;

impl Backend for NiriBackend {
    fn is_available() -> bool {
        Socket::connect().is_ok()
    }

    fn subscribe(&mut self) -> mpsc::Receiver<spyland_core::Event> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            run(tx);
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

fn run(tx: mpsc::Sender<Event>) {
    let mut event_socket = Socket::connect().expect("failed to connect to niri");
    let mut query_socket = Socket::connect().expect("failed to connect to niri");

    let reply = event_socket
        .send(Request::EventStream)
        .expect("failed to send request to niri");

    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = event_socket.read_events();
        loop {
            match read_event() {
                Ok(event) => {
                    match event {
                        NiriEvent::WindowFocusChanged { id } => {
                            if let Some(id) = id {
                                // TODO: Window cache?
                                let window = resolve_window(&mut query_socket, id);
                                if let Some(w) = window {
                                    if tx.send(Event::ActiveWindowChanged(w.app_id)).is_err() {
                                        break;
                                    }
                                }
                            } else {
                                // TODO: Idle
                                // tx.send(Event::Idle(true));
                            }
                        }
                        NiriEvent::WorkspaceActivated { id, focused } => {
                            if focused {
                                if tx
                                    .send(Event::WorkspaceChanged(id.try_into().unwrap()))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("niri event error: {:?}", e);
                    continue;
                }
            }
        }
    }
}
