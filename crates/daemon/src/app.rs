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
use log::{debug, info, trace, warn};
use tokio::time::interval;

pub struct App<C: Clock> {
    session_manager: Arc<Mutex<SessionManager<C>>>,
    db: Db,
    server: IpcServer,
}

impl<C: Clock> App<C> {
    pub async fn new(db: Db, server: IpcServer, config: &str, clock: C) -> Result<Self> {
        let toml: toml::Value = toml::from_str(config)?;

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

            let (response, session) = {
                let mut sm_lock = session_manager.lock().unwrap();
                let response = sm_lock.handle_event(Event::Tick);
                let session = if let Response::Flushed { .. } = response {
                    sm_lock.sessions().last().cloned()
                } else {
                    None
                };
                (response, session)
            };

            if let Response::Flushed { merged } = response
                && let Some(session) = session
            {
                if !merged {
                    database.insert(session.into()).await
                } else {
                    database.update_last(session.into()).await
                }
                .expect("Database operation failed");
            }
        }
    }

    fn ipc_server(mut server: IpcServer, session_manager: Arc<Mutex<SessionManager<C>>>) {
        trace!("ipc_server()");

        info!("Waiting for backend...");

        loop {
            let conn = server.accept().expect("Accept new connection failed");

            let sm = session_manager.clone();
            tokio::task::spawn_blocking(move || {
                Self::connection_handler(conn, sm);
            });
        }
    }

    fn connection_handler(conn: IpcConnection, session_manager: Arc<Mutex<SessionManager<C>>>) {
        loop {
            match conn.read() {
                Ok(request) => {
                    let response = match request {
                        IpcRequest::Ping => IpcResponse::Pong,
                        IpcRequest::Handshake {
                            protocol_version,
                            backend_name,
                        } => {
                            use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};
                            let is_accepted = protocol_version <= protocol::VERSION;
                            let pid = match getsockopt(conn.stream(), PeerCredentials) {
                                Ok(cred) => cred.pid().to_string(),
                                Err(e) => e.desc().to_owned(),
                            };
                            match is_accepted {
                                true => {
                                    info!(
                                        "The '{backend_name}' ({pid}) backend is accepted! Protocol version: {protocol_version}"
                                    )
                                }
                                false => {
                                    warn!(
                                        "The '{backend_name}' ({pid}) backend was rejected due to version incompatibility.",
                                    );
                                    debug!("{protocol_version} > {}", protocol::VERSION);
                                }
                            }

                            IpcResponse::Handshake {
                                protocol_version: protocol::VERSION,
                                is_accepted,
                            }
                        }
                        IpcRequest::Event(event) => IpcResponse::EventResponse(
                            session_manager.lock().unwrap().handle_event(event),
                        ),
                    };

                    conn.send(response).expect("Failed to send response");
                }
                Err(err) => {
                    warn!("Read failed: '{err:#}'. Connection will be shutdown.");
                    break;
                }
            }
        }
    }
}
