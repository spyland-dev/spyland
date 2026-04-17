/*
 *  spylandd — background daemon for continuous screen time tracking
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    time::Duration,
};

use spyland_backend_niri::NiriBackend;
use spyland_core::{Backend, Clock, Event, Response, SessionManager};

use anyhow::{Context, Result};
use tokio::time::interval;

use spyland_lib::db::Db;

pub struct App<C: Clock> {
    receiver: Receiver<Event>,
    session_manager: Arc<Mutex<SessionManager<C>>>,
    db: Db,
}

impl<C: Clock> App<C> {
    pub async fn new(db: Db, clock: C) -> Result<Self> {
        let mut backend = new_backend().context("No backend is available")?;

        db.create().await.context("Failed to create database")?;

        Ok(Self {
            receiver: backend.subscribe(),
            session_manager: Arc::new(Mutex::new(SessionManager::new(clock))),
            db,
        })
    }
}

impl<C: Clock + Send + 'static> App<C> {
    pub async fn run(self) -> Result<()> {
        let sm = self.session_manager.clone();
        let rx = self.receiver;
        let event_task = tokio::task::spawn_blocking(move || {
            Self::event_handler(sm, rx);
        });

        let sm = self.session_manager.clone();
        let db = self.db;
        let tick_task = tokio::task::spawn_local(async move {
            Self::tick_handler(sm, db).await;
        });

        tokio::try_join!(event_task, tick_task)?;

        Ok(())
    }

    fn event_handler(session_manager: Arc<Mutex<SessionManager<C>>>, receiver: Receiver<Event>) {
        for event in receiver {
            session_manager.lock().unwrap().handle_event(event);
        }
    }

    async fn tick_handler(session_manager: Arc<Mutex<SessionManager<C>>>, database: Db) {
        let mut timer = interval(Duration::from_secs(1));

        loop {
            timer.tick().await;
            let mut sm_lock = session_manager.lock().unwrap();
            let response = sm_lock.handle_event(Event::Tick);

            if matches!(response, Response::Flush)
                && let Some(session) = sm_lock.sessions().last()
            {
                database
                    .insert(session.clone().into())
                    .await
                    .expect("Write to database failed");
            }
        }
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
