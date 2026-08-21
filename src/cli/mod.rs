use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::config::Config;
use crate::push::{self, FcmClient};

#[derive(Debug, Parser)]
#[command(name = "power-monitor", about = "Linux power monitoring daemon")]
pub struct Cli {
    /// Path to config.toml (default: ~/.config/power-monitor/config.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the background daemon (default)
    Daemon,
    /// Show current power status via local API
    Status,
    /// Show recent power events
    Events {
        #[arg(long, default_value_t = 20)]
        last: u32,
    },
    /// Print effective configuration path and values
    Config,
    /// Manage Firebase Cloud Messaging device tokens
    PushToken {
        #[command(subcommand)]
        action: PushTokenAction,
    },
    /// Print version
    Version,
}

#[derive(Debug, Subcommand)]
pub enum PushTokenAction {
    /// Register a device token
    Add { token: String },
    /// List registered tokens
    List,
    /// Remove a device token
    Remove { token: String },
}

pub async fn status() -> Result<()> {
    let cfg = Config::load(None)?;
    let url = format!("{}/api/v1/power", cfg.api_base());
    let body = http_get(&url).await?;
    let ac = body
        .get("ac_connected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let pct = body
        .get("battery_percentage")
        .and_then(|v| v.as_f64())
        .map(|p| format!("{}%", p.round() as u8))
        .unwrap_or_else(|| "n/a".into());
    let state = body
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let health = body
        .get("battery_health")
        .and_then(|v| v.as_f64())
        .map(|h| format!("{}%", h.round() as u8))
        .unwrap_or_else(|| "n/a".into());

    println!("Power:       {}", if ac { "AC Connected" } else { "On Battery" });
    println!("Battery:     {pct}");
    println!("State:       {state}");
    println!("Health:      {health}");
    Ok(())
}

pub async fn events(last: u32) -> Result<()> {
    let cfg = Config::load(None)?;
    let url = format!("{}/api/v1/events?page=1&limit={last}", cfg.api_base());
    let body = http_get(&url).await?;
    let Some(items) = body.get("events").and_then(|v| v.as_array()) else {
        bail!("unexpected events response");
    };
    for e in items {
        let ts = e.get("timestamp").and_then(|v| v.as_str()).unwrap_or("?");
        let kind = e.get("event").and_then(|v| v.as_str()).unwrap_or("?");
        let pct = e
            .get("battery_percentage")
            .and_then(|v| v.as_f64())
            .map(|p| format!("{}%", p.round() as u8))
            .unwrap_or_else(|| "n/a".into());
        println!("{ts}  {kind:<28}  Battery: {pct}");
    }
    Ok(())
}

pub fn show_config() -> Result<()> {
    let path = Config::default_config_path();
    let cfg = Config::load(None)?;
    println!("config_path: {}", path.display());
    println!("data_dir:    {}", cfg.data_dir().display());
    println!();
    print!("{}", toml::to_string_pretty(&cfg)?);
    Ok(())
}

pub fn push_token(action: PushTokenAction) -> Result<()> {
    let cfg = Config::load(None)?;
    let path = push::default_tokens_path();
    let client = FcmClient::from_env(&cfg.data_dir())?;
    let path = client
        .as_ref()
        .map(|c| c.tokens_path().to_path_buf())
        .unwrap_or(path);

    match action {
        PushTokenAction::Add { token } => {
            if let Some(client) = &client {
                client.add_token(&token)?;
                let n = client.load_tokens()?.len();
                println!("saved {n} token(s) -> {}", client.tokens_path().display());
            } else {
                let mut tokens = push::load_tokens_file(&path)?;
                let token = token.trim().to_string();
                if token.is_empty() {
                    bail!("empty token");
                }
                if !tokens.iter().any(|t| t == &token) {
                    tokens.push(token);
                    push::save_tokens_file(&path, &tokens)?;
                }
                println!("saved {} token(s) -> {}", tokens.len(), path.display());
            }
        }
        PushTokenAction::List => {
            let tokens = push::load_tokens_file(&path)?;
            println!("file: {}", path.display());
            if tokens.is_empty() {
                println!("(no tokens)");
            } else {
                for t in tokens {
                    println!("{t}");
                }
            }
        }
        PushTokenAction::Remove { token } => {
            if let Some(client) = &client {
                let removed = client.remove_token(&token)?;
                let n = client.load_tokens()?.len();
                println!(
                    "removed {}; {} remaining -> {}",
                    if removed { 1 } else { 0 },
                    n,
                    client.tokens_path().display()
                );
            } else {
                let mut tokens = push::load_tokens_file(&path)?;
                let before = tokens.len();
                tokens.retain(|t| t != &token);
                push::save_tokens_file(&path, &tokens)?;
                println!(
                    "removed {}; {} remaining -> {}",
                    before.saturating_sub(tokens.len()),
                    tokens.len(),
                    path.display()
                );
            }
        }
    }
    Ok(())
}

async fn http_get(url: &str) -> Result<Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let cfg = Config::load(None)?;
    let host = cfg.server.host.clone();
    let port = cfg.server.port;
    let path = url
        .strip_prefix(&cfg.api_base())
        .unwrap_or("/")
        .to_string();

    let mut stream = TcpStream::connect((host.as_str(), port))
        .await
        .with_context(|| {
            format!(
                "cannot connect to power-monitor API at {}:{} (is the daemon running?)",
                host, port
            )
        })?;

    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    let text = String::from_utf8_lossy(&buf);
    let Some(idx) = text.find("\r\n\r\n") else {
        bail!("invalid HTTP response from daemon");
    };
    let body = &text[idx + 4..];
    let status_line = text.lines().next().unwrap_or("");
    if !status_line.contains("200") {
        bail!("API error: {status_line}");
    }
    serde_json::from_str(body).context("parsing API JSON")
}
