/*
 *  spylandd — background daemon for continuous screen time tracking
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use spyland_core::{
    Event,
    manager::{Clock, Configuration as CoreConfig, Response, SessionManager},
};
use spyland_lib::{
    db::Db,
    ipc::{
        IpcConnection, IpcServer,
        protocol::{self, Request as IpcRequest, Response as IpcResponse},
    },
};

use anyhow::{Context, Result};
use log::{info, trace, warn};
use tokio::time::interval;

pub struct App<C: Clock> {
    session_manager: Arc<Mutex<SessionManager<C>>>,
    db: Db,
    server: IpcServer,
}

impl<C: Clock> App<C> {
    pub async fn new(db: Db, server: IpcServer, config: &str, clock: C) -> Result<Self> {
        let toml: toml::Value = toml::from_str(&config)?;

        let mut sm = SessionManager::new(clock);
        let config: CoreConfig = match toml.get("core") {
            Some(value) => value
                .clone()
                .try_into()
                .context("Failed to deserialize `core` section from config")?,

            None => {
                warn!("Failed to get `core` section from the config! Use default.");
                CoreConfig::default()
            }
        };
        sm.set_config(config);

        db.create().await.context("Failed to create database")?;

        Ok(Self {
            session_manager: Arc::new(Mutex::new(sm)),
            server,
            db,
        })
    }
}

impl<C: Clock + Send + 'static> App<C> {
    pub async fn run(self) -> Result<()> {
        let sm = self.session_manager.clone();
        let db = self.db;
        let tick_task = tokio::task::spawn_local(async move {
            Self::tick_handler(sm, db).await;
        });

        let sm = self.session_manager.clone();
        let sv = self.server;
        let ipc_task = tokio::task::spawn_blocking(move || Self::ipc_server(sv, sm));
        tokio::try_join!(tick_task, ipc_task)?;

        Ok(())
    }

    async fn tick_handler(session_manager: Arc<Mutex<SessionManager<C>>>, database: Db) {
        trace!("tick_handler()");

        let mut timer = interval(Duration::from_secs(1));
        loop {
            timer.tick().await;
            let mut sm_lock = session_manager.lock().unwrap();
            let response = sm_lock.handle_event(Event::Tick);

            if let Response::Flushed { merged } = response
                && let Some(session) = sm_lock.sessions().last()
            {
                if !merged {
                    database.insert(session.clone().into()).await
                } else {
                    database.update_last(session.clone().into()).await
                }
                .expect("Database operation failed");
            }
        }
    }

    fn ipc_server(mut server: IpcServer, session_manager: Arc<Mutex<SessionManager<C>>>) {
        trace!("ipc_server()");

        loop {
            let conn = server.accept().expect("Accept new connection failed");

            let sm = session_manager.clone();
            tokio::task::spawn_blocking(move || {
                Self::connection_handler(conn, sm);
            });
        }
    }

    fn connection_handler(conn: IpcConnection, session_manager: Arc<Mutex<SessionManager<C>>>) {
        while let Ok(request) = conn.read() {
            let response = match request {
                IpcRequest::Ping => IpcResponse::Pong,
                IpcRequest::Handshake {
                    protocol_version,
                    backend_name,
                } => {
                    info!(
                        "New backend wants establish the connection! '{backend_name}' ({protocol_version})"
                    );
                    IpcResponse::Handshake {
                        protocol_version: protocol::VERSION,
                    }
                }
                IpcRequest::Event(event) => {
                    IpcResponse::EventResponse(session_manager.lock().unwrap().handle_event(event))
                }
            };

            conn.send(response).expect("Failed to send response");
        }
    }
}
