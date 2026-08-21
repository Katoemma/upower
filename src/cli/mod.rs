use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::auth;
use crate::config::Config;
use crate::database::EventStore;
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
    /// Login and save API token for CLI use
    Login {
        email: String,
        password: String,
    },
    /// Seeded user management (no public signup)
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    /// Manage Firebase Cloud Messaging device tokens
    PushToken {
        #[command(subcommand)]
        action: PushTokenAction,
    },
    /// Print version
    Version,
}

#[derive(Debug, Subcommand)]
pub enum UserAction {
    /// Add a seeded user (email + password)
    Add {
        email: String,
        password: String,
    },
    /// List seeded users
    List,
    /// Remove a user
    Remove { email: String },
    /// Reset a user's password
    SetPassword {
        email: String,
        password: String,
    },
    /// Enable/disable email alerts for a user (`true`/`false`)
    NotifyEmail {
        email: String,
        enabled: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PushTokenAction {
    /// Register a device token (requires login; attaches to your user via API)
    Add { token: String },
    /// List tokens for the logged-in user (API) or local file fallback
    List,
    /// Remove a device token
    Remove { token: String },
}

pub async fn status() -> Result<()> {
    let cfg = Config::load(None)?;
    let url = format!("{}/api/v1/power", cfg.api_base());
    let body = http_json("GET", &url, None).await?;
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
    let body = http_json("GET", &url, None).await?;
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

pub async fn login_cmd(email: String, password: String) -> Result<()> {
    let cfg = Config::load(None)?;
    let url = format!("{}/api/v1/auth/login", cfg.api_base());
    let body = serde_json::json!({ "email": email, "password": password });
    let resp = http_json("POST", &url, Some(body)).await?;
    let Some(token) = resp.get("token").and_then(|v| v.as_str()) else {
        bail!("login response missing token: {resp}");
    };
    auth::save_cli_token(token)?;
    let email = resp
        .pointer("/user/email")
        .and_then(|v| v.as_str())
        .unwrap_or(&email);
    println!("logged in as {email}");
    println!("token saved to {}", auth::token_file_path().display());
    Ok(())
}

pub async fn user_cmd(action: UserAction) -> Result<()> {
    let cfg = Config::load(None)?;
    let store = EventStore::open(&cfg.data_dir()).await?;

    match action {
        UserAction::Add { email, password } => {
            let user = store.create_user(&email, &password).await?;
            println!("created user id={} email={}", user.id, user.email);
            println!("restart the daemon so API auth picks up the new user count");
        }
        UserAction::List => {
            let users = store.list_users().await?;
            if users.is_empty() {
                println!("(no users — API auth is open until you seed one)");
            } else {
                for u in users {
                    println!(
                        "{:<4}  {:<40}  notify_email={}",
                        u.id, u.email, u.notify_email
                    );
                }
            }
        }
        UserAction::Remove { email } => {
            if store.remove_user(&email).await? {
                println!("removed {email}");
            } else {
                bail!("user not found: {email}");
            }
        }
        UserAction::SetPassword { email, password } => {
            if store.set_password(&email, &password).await? {
                println!("password updated for {email}");
            } else {
                bail!("user not found: {email}");
            }
        }
        UserAction::NotifyEmail { email, enabled } => {
            let enabled = match enabled.to_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => true,
                "0" | "false" | "no" | "off" => false,
                other => bail!("expected true/false for enabled, got {other}"),
            };
            if store.set_notify_email(&email, enabled).await? {
                println!("notify_email={enabled} for {email}");
            } else {
                bail!("user not found: {email}");
            }
        }
    }
    Ok(())
}

pub async fn push_token(action: PushTokenAction) -> Result<()> {
    let cfg = Config::load(None)?;

    match action {
        PushTokenAction::Add { token } => {
            // Prefer API (attaches to logged-in user).
            let url = format!("{}/api/v1/push/tokens", cfg.api_base());
            let body = serde_json::json!({ "token": token });
            match http_json("POST", &url, Some(body)).await {
                Ok(resp) => {
                    println!("registered via API: {resp}");
                    return Ok(());
                }
                Err(err) => {
                    eprintln!("API register failed ({err}); falling back to local file");
                }
            }
            let path = push::default_tokens_path();
            let client = FcmClient::from_env(&cfg.data_dir())?;
            if let Some(client) = client {
                client.add_token(&token)?;
                println!(
                    "saved {} token(s) -> {}",
                    client.load_tokens()?.len(),
                    client.tokens_path().display()
                );
            } else {
                let mut tokens = push::load_tokens_file(&path)?;
                if !tokens.iter().any(|t| t == &token) {
                    tokens.push(token);
                    push::save_tokens_file(&path, &tokens)?;
                }
                println!("saved {} token(s) -> {}", tokens.len(), path.display());
            }
        }
        PushTokenAction::List => {
            let url = format!("{}/api/v1/push/tokens", cfg.api_base());
            if let Ok(resp) = http_json("GET", &url, None).await {
                println!("{resp}");
                return Ok(());
            }
            let path = FcmClient::from_env(&cfg.data_dir())?
                .map(|c| c.tokens_path().to_path_buf())
                .unwrap_or_else(push::default_tokens_path);
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
            let cfg = Config::load(None)?;
            let store = EventStore::open(&cfg.data_dir()).await?;
            let removed = store.remove_fcm_token(&token).await?;
            let path = push::default_tokens_path();
            if let Ok(client) = FcmClient::from_env(&cfg.data_dir()) {
                if let Some(client) = client {
                    let _ = client.remove_token(&token);
                }
            } else if path.exists() {
                let mut tokens = push::load_tokens_file(&path)?;
                tokens.retain(|t| t != &token);
                let _ = push::save_tokens_file(&path, &tokens);
            }
            println!("removed from db: {removed}");
        }
    }
    Ok(())
}

async fn http_json(method: &str, url: &str, body: Option<Value>) -> Result<Value> {
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

    let payload = body
        .as_ref()
        .map(|b| b.to_string())
        .unwrap_or_default();
    let auth_header = auth::load_cli_token()
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();

    let req = if method == "GET" {
        format!(
            "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: application/json\r\n{auth_header}\r\n"
        )
    } else {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth_header}\r\n{payload}",
            payload.len()
        )
    };
    stream.write_all(req.as_bytes()).await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    let text = String::from_utf8_lossy(&buf);
    let Some(idx) = text.find("\r\n\r\n") else {
        bail!("invalid HTTP response from daemon");
    };
    let body = &text[idx + 4..];
    let status_line = text.lines().next().unwrap_or("");
    if !(status_line.contains("200") || status_line.contains("201")) {
        bail!("API error: {status_line} — {body}");
    }
    if body.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(body).context("parsing API JSON")
}
