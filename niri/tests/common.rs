/*
 *  spyland-backend-niri — niri Wayland compositor integration
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration,
};

use niri_ipc::{Reply, Request, Response, Window};
use spyland_backend_niri::NiriBackend;
use spyland_core::Backend;
use tempfile::{Builder, NamedTempFile};

pub use niri_ipc::Event as NiriEvent;
pub use spyland_core::Event as CoreEvent;

pub struct FakeNiriServer {
    socket_path: NamedTempFile<()>,
    listener: UnixListener,
    windows: Arc<Mutex<Vec<Window>>>,
    ev_sender: Sender<NiriEvent>,
    ev_receiver: Arc<Mutex<Receiver<NiriEvent>>>,
}

impl FakeNiriServer {
    pub fn new() -> Self {
        let socket_path = Builder::new().make(|_| Ok(())).unwrap();

        let listener = UnixListener::bind(&socket_path).expect("failed to bind socket");
        let (ev_sender, ev_receiver) = mpsc::channel();
        Self {
            socket_path,
            listener,
            windows: Arc::new(Mutex::new(Vec::new())),
            ev_sender,
            ev_receiver: Arc::new(Mutex::new(ev_receiver)),
        }
    }

    pub fn run(&self) {
        while self.socket_path.path().exists() {
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    let mut reader = BufReader::new(&stream);
                    let mut line = String::new();

                    if reader.read_line(&mut line).is_err() {
                        return;
                    }

                    let writer = stream.try_clone().expect("failed to clone stream");
                    let request: Result<Request, _> = serde_json::from_str(&line);

                    match request {
                        Ok(Request::EventStream) => {
                            let rx = self.ev_receiver.clone();
                            thread::spawn(move || Self::handle_event_stream(writer, rx));
                        }
                        _ => {}
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn handle_event_stream(mut writer: UnixStream, rx: Arc<Mutex<Receiver<NiriEvent>>>) {
        let reply: Reply = Ok(Response::Handled);
        if let Ok(json) = serde_json::to_string(&reply) {
            let _ = writeln!(writer, "{}", json);
        }
        for event in rx.lock().unwrap().iter() {
            if let Ok(json) = serde_json::to_string(&event) {
                if writeln!(writer, "{}", json).is_err() {
                    break;
                }

                if writer.flush().is_err() {
                    break;
                }
            }
        }
    }
}

pub struct TestDriver {
    server: Arc<Mutex<FakeNiriServer>>,
    receiver: Receiver<CoreEvent>,
}

impl TestDriver {
    pub fn new() -> Self {
        let _ = env_logger::try_init();

        let server = Arc::new(Mutex::new(FakeNiriServer::new()));

        let mut backend = NiriBackend::new(
            server
                .lock()
                .unwrap()
                .socket_path
                .path()
                .to_path_buf()
                .clone(),
        );
        let receiver = backend.subscribe();

        let server_clone = server.clone();
        thread::spawn(move || server_clone.lock().unwrap().run());
        Self { server, receiver }
    }

    pub fn send(&self, event: NiriEvent) {
        self.server
            .lock()
            .unwrap()
            .ev_sender
            .send(event)
            .expect("failed to send event");
    }

    pub fn new_test_window(&mut self) -> (u64, String) {
        let server = self.server.lock().unwrap();
        let mut windows = server.windows.lock().unwrap();

        let id: u64 = windows.len().try_into().unwrap();
        let app_id = format!("test_app_{id}");

        use niri_ipc::WindowLayout;
        let window = Window {
            id: id,
            title: Some(format!("Test Window {}", id)),
            app_id: Some(app_id.clone()),
            pid: Some(1000),
            workspace_id: Some(0),
            is_focused: false,
            is_floating: false,
            is_urgent: false,
            layout: WindowLayout {
                pos_in_scrolling_layout: None,
                tile_size: (1920.0, 1080.0),
                window_size: (1920, 1080),
                tile_pos_in_workspace_view: None,
                window_offset_in_tile: (0.0, 0.0),
            },
            focus_timestamp: None,
        };

        windows.push(window.clone());
        server
            .ev_sender
            .send(NiriEvent::WindowOpenedOrChanged { window })
            .expect("failed to send event");
        server
            .ev_sender
            .send(NiriEvent::WindowFocusChanged { id: Some(id) })
            .expect("failed to send event");

        (id, app_id)
    }

    pub fn assert_event(&self, event: CoreEvent) {
        assert_eq!(
            event,
            self.receiver
                .recv_timeout(Duration::from_millis(1000))
                .expect("receive failed")
        );
    }
}
