//! Power Monitor — Linux-native power monitoring daemon.

mod api;
mod auth;
mod cli;
mod config;
mod database;
mod email;
mod logging;
mod notifications;
mod power;
mod push;
mod upower;
mod users;
mod websocket;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};

use crate::auth::AuthState;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::database::EventStore;
use crate::email::SmtpSettings;
use crate::power::{PowerEvent, PowerMonitor, PowerState};
use crate::push::FcmClient;
use crate::upower::UPowerClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => cli::status().await,
        Some(Commands::Events { last }) => cli::events(last).await,
        Some(Commands::Config) => cli::show_config(),
        Some(Commands::PushToken { action }) => cli::push_token(action).await,
        Some(Commands::User { action }) => cli::user_cmd(action).await,
        Some(Commands::Login { email, password }) => cli::login_cmd(email, password).await,
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
    let smtp = SmtpSettings::from_env().context("loading SMTP settings from environment")?;
    match &smtp {
        Some(s) => info!(
            host = %s.host,
            to = %s.to,
            "email notifications enabled (Brevo/SMTP)"
        ),
        None => info!("email notifications disabled (no SMTP_* env vars)"),
    }

    let fcm = FcmClient::from_env(&config.data_dir()).context("loading Firebase/FCM settings")?;
    match &fcm {
        Some(client) => info!(
            tokens = %client.tokens_path().display(),
            "Firebase push enabled"
        ),
        None => info!(
            "Firebase push disabled (set FIREBASE_CREDENTIALS or FCM_SERVER_KEY)"
        ),
    }

    info!(
        host = %config.server.host,
        port = config.server.port,
        "starting power-monitor daemon"
    );

    let store = EventStore::open(&config.data_dir())
        .await
        .context("failed to open event database")?;
    let user_count = store.user_count().await.unwrap_or(0);
    let auth = AuthState::load(&config.data_dir(), user_count)
        .context("loading auth state")?;
    if auth.required {
        info!(users = user_count, "API authentication required (seeded users present)");
    } else {
        info!("API authentication open (no seeded users yet)");
    }
    let store = Arc::new(store);

    let state = Arc::new(RwLock::new(PowerState::default()));
    let (event_tx, _) = broadcast::channel::<PowerEvent>(256);

    let client = UPowerClient::connect()
        .await
        .context("failed to connect to UPower over D-Bus")?;

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
    let email_config = config.email.clone();
    let push_config = config.push.clone();
    let battery_config = config.battery.clone();

    tokio::spawn(async move {
        let mut monitor = PowerMonitor::new(
            client,
            monitor_state,
            monitor_tx,
            monitor_store,
            monitor_config,
            notif_config,
            email_config,
            smtp,
            push_config,
            fcm,
            battery_config,
        );
        if let Err(err) = monitor.run().await {
            error!(error = %err, "power monitor exited with error");
        }
    });

    let app = api::router(
        Arc::clone(&state),
        Arc::clone(&store),
        event_tx.clone(),
        config.data_dir(),
        auth,
    );
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
