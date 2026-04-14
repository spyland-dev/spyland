/*
 *  spyland — screen time for Wayland
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  Licensed under the GNU General Public License v3.0
 */

use std::sync::mpsc::Receiver;

use spyland_backend_niri::NiriBackend;
use spyland_core::{Backend, Clock, Event, Response, SessionManager};

use anyhow::{Context, Result};

use crate::db::Db;

pub struct App<C: Clock> {
    receiver: Receiver<Event>,
    session_manager: SessionManager<C>,
    db: Db,
}

impl<C: Clock> App<C> {
    pub async fn new(db: Db, clock: C) -> Result<Self> {
        let mut backend = new_backend().context("No backend is available")?;

        db.create().await.context("Failed to create database")?;

        Ok(Self {
            receiver: backend.subscribe(),
            session_manager: SessionManager::new(clock),
            db,
        })
    }

    pub async fn event_handler(mut self) -> Result<()> {
        for event in self.receiver {
            println!("{:?}", event);
            let response = self.session_manager.handle_event(event);

            if matches!(response, Response::Flush) {
                let session = self.session_manager.sessions().last().unwrap();

                self.db.insert(session.clone()).await?;
            }
        }

        Ok(())
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
