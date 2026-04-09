/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::time::{SystemTime, UNIX_EPOCH};

use log::info;

use spyland_backend_niri::NiriBackend;
use spyland_core::{Backend, Clock, SessionManager};

fn main() {
    env_logger::init();

    info!("Starting spyland daemon...");

    let mut backend = new_backend().expect("no backend is available");
    let receiver = backend.subscribe();
    let system_clock = SystemClock {};
    let mut session_manager = SessionManager::new(system_clock);

    for event in receiver {
        println!("{:?}", event);
        session_manager.handle_event(event);
    }
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs()
    }
}

fn new_backend() -> Option<Box<dyn Backend>> {
    let backends: Vec<Box<dyn Backend>> = vec![Box::new(NiriBackend::default())];

    for backend in backends {
        if backend.is_available() {
            return Some(backend);
        }
    }

    None
}
