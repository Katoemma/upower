//! Power Monitor — Linux-native power monitoring daemon.

mod api;
mod cli;
mod config;
mod database;
mod logging;
mod notifications;
mod power;
mod upower;
mod websocket;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::database::EventStore;
use crate::power::{PowerEvent, PowerMonitor, PowerState};
use crate::upower::UPowerClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => cli::status().await,
        Some(Commands::Events { last }) => cli::events(last).await,
        Some(Commands::Config) => cli::show_config(),
        Some(Commands::Version) => {
            println!("power-monitor {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Commands::Daemon) | None => run_daemon(cli.config).await,
    }
}

async fn run_daemon(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    logging::init();

    let config = Config::load(config_path.as_deref())
        .context("failed to load configuration")?;
    info!(
        host = %config.server.host,
        port = config.server.port,
        "starting power-monitor daemon"
    );

    let store = EventStore::open(&config.data_dir())
        .await
        .context("failed to open event database")?;
    let store = Arc::new(store);

    let state = Arc::new(RwLock::new(PowerState::default()));
    let (event_tx, _) = broadcast::channel::<PowerEvent>(256);

    let client = UPowerClient::connect()
        .await
        .context("failed to connect to UPower over D-Bus")?;

    // Initial snapshot
    let snapshot = client
        .read_snapshot()
        .await
        .context("failed to read initial power snapshot")?;
    {
        let mut guard = state.write().await;
        *guard = snapshot;
    }
    info!(ac = ?state.read().await.ac_connected, "initial power state loaded");

    let monitor_state = Arc::clone(&state);
    let monitor_tx = event_tx.clone();
    let monitor_store = Arc::clone(&store);
    let monitor_config = config.clone();
    let notif_config = config.notifications.clone();
    let battery_config = config.battery.clone();

    tokio::spawn(async move {
        let mut monitor = PowerMonitor::new(
            client,
            monitor_state,
            monitor_tx,
            monitor_store,
            monitor_config,
            notif_config,
            battery_config,
        );
        if let Err(err) = monitor.run().await {
            error!(error = %err, "power monitor exited with error");
        }
    });

    let app = api::router(Arc::clone(&state), Arc::clone(&store), event_tx.clone());
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind API on {addr}"))?;
    info!(%addr, "API listening (localhost-only by default)");

    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}
