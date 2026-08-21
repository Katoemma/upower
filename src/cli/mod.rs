use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::config::Config;

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
    /// Print version
    Version,
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

async fn http_get(url: &str) -> Result<Value> {
    // Minimal HTTP GET without pulling in reqwest — use tokio TCP + manual request,
    // or use std via blocking. Prefer a tiny approach with ureq... but we didn't add ureq.
    // Use hyper via axum's dependency? Simpler: use std::process curl, or add reqwest.
    // Use tokio::net + write raw HTTP for GET on localhost.
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
