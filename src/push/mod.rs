//! Firebase Cloud Messaging (FCM) push notifications.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::PushConfig;
use crate::power::PowerEvent;

#[derive(Debug, Clone)]
pub struct FcmClient {
    auth: FcmAuth,
    tokens_path: PathBuf,
    /// Cached OAuth access token for HTTP v1.
    token_cache: Arc<Mutex<Option<CachedToken>>>,
}

#[derive(Debug, Clone)]
enum FcmAuth {
    /// Legacy API (server key). Simpler for testing; prefer V1 for production.
    Legacy { server_key: String },
    /// FCM HTTP v1 with a Firebase service-account JSON.
    V1 {
        project_id: String,
        client_email: String,
        private_key_pem: String,
    },
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    project_id: String,
    client_email: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
struct GoogleClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: u64,
    exp: u64,
}

impl FcmClient {
    /// Load from env. Returns `Ok(None)` when Firebase is not configured.
    pub fn from_env(data_dir: &Path) -> Result<Option<Self>> {
        let tokens_path = env::var("FCM_TOKENS_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("fcm_tokens.txt"));

        if let Some(path) = env::var("FIREBASE_CREDENTIALS")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                env::var("GOOGLE_APPLICATION_CREDENTIALS")
                    .ok()
                    .filter(|s| !s.is_empty())
            })
        {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("reading Firebase credentials {path}"))?;
            let sa: ServiceAccount =
                serde_json::from_str(&text).context("parsing Firebase service account JSON")?;
            let project_id = env::var("FIREBASE_PROJECT_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or(sa.project_id);
            return Ok(Some(Self {
                auth: FcmAuth::V1 {
                    project_id,
                    client_email: sa.client_email,
                    private_key_pem: sa.private_key,
                },
                tokens_path,
                token_cache: Arc::new(Mutex::new(None)),
            }));
        }

        if let Some(server_key) = env::var("FCM_SERVER_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| env::var("FIREBASE_SERVER_KEY").ok().filter(|s| !s.is_empty()))
        {
            return Ok(Some(Self {
                auth: FcmAuth::Legacy { server_key },
                tokens_path,
                token_cache: Arc::new(Mutex::new(None)),
            }));
        }

        Ok(None)
    }

    pub fn tokens_path(&self) -> &Path {
        &self.tokens_path
    }

    pub fn load_tokens(&self) -> Result<Vec<String>> {
        load_tokens_file(&self.tokens_path)
    }

    pub fn add_token(&self, token: &str) -> Result<()> {
        let token = token.trim();
        if token.is_empty() {
            bail!("empty FCM token");
        }
        let mut tokens = self.load_tokens()?;
        if !tokens.iter().any(|t| t == token) {
            tokens.push(token.to_string());
            save_tokens_file(&self.tokens_path, &tokens)?;
        }
        Ok(())
    }

    pub fn remove_token(&self, token: &str) -> Result<bool> {
        let mut tokens = self.load_tokens()?;
        let before = tokens.len();
        tokens.retain(|t| t != token);
        if tokens.len() == before {
            return Ok(false);
        }
        save_tokens_file(&self.tokens_path, &tokens)?;
        Ok(true)
    }

    pub fn maybe_send(&self, cfg: &PushConfig, event: &PowerEvent) {
        let tokens = match self.load_tokens() {
            Ok(t) => t,
            Err(err) => {
                warn!(error = %err, "failed to load FCM tokens");
                return;
            }
        };
        self.maybe_send_to(cfg, event, &tokens);
    }

    pub fn maybe_send_to(&self, cfg: &PushConfig, event: &PowerEvent, tokens: &[String]) {
        if !cfg.enabled {
            return;
        }
        let Some((title, body)) = crate::notifications::message_for_event(
            event,
            cfg.ac_connected,
            cfg.ac_disconnected,
            cfg.fully_charged,
            cfg.low_battery,
            cfg.critical_battery,
        ) else {
            return;
        };

        if tokens.is_empty() {
            warn!("push enabled but no FCM device tokens registered");
            return;
        }

        for token in tokens {
            match self.send_to_token(token, &title, &body, event) {
                Ok(()) => info!(token_prefix = %truncate(token), "FCM push sent"),
                Err(err) => warn!(
                    error = %err,
                    token_prefix = %truncate(token),
                    "FCM push failed"
                ),
            }
        }
    }

    fn send_to_token(
        &self,
        token: &str,
        title: &str,
        body: &str,
        event: &PowerEvent,
    ) -> Result<()> {
        match &self.auth {
            FcmAuth::Legacy { server_key } => {
                send_legacy(server_key, token, title, body, event)
            }
            FcmAuth::V1 {
                project_id,
                client_email,
                private_key_pem,
            } => {
                let access = self.oauth_token(client_email, private_key_pem)?;
                send_v1(project_id, &access, token, title, body, event)
            }
        }
    }

    fn oauth_token(&self, client_email: &str, private_key_pem: &str) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        {
            let cache = self.token_cache.lock().unwrap();
            if let Some(cached) = cache.as_ref() {
                if cached.expires_at > now + 60 {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let claims = GoogleClaims {
            iss: client_email.to_string(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".into(),
            aud: "https://oauth2.googleapis.com/token".into(),
            iat: now,
            exp: now + 3600,
        };
        let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .context("invalid Firebase private_key PEM")?;
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key)
            .context("signing Google OAuth JWT")?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?;
        let resp: serde_json::Value = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", jwt.as_str()),
            ])
            .send()
            .context("Google OAuth token request")?
            .error_for_status()
            .context("Google OAuth token HTTP error")?
            .json()
            .context("parsing Google OAuth token response")?;

        let access_token = resp
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("OAuth response missing access_token: {resp}"))?
            .to_string();
        let expires_in = resp
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(3600);

        let mut cache = self.token_cache.lock().unwrap();
        *cache = Some(CachedToken {
            access_token: access_token.clone(),
            expires_at: now + expires_in,
        });
        Ok(access_token)
    }
}

fn send_legacy(
    server_key: &str,
    token: &str,
    title: &str,
    body: &str,
    event: &PowerEvent,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;
    let payload = serde_json::json!({
        "to": token,
        "priority": "high",
        "notification": {
            "title": title,
            "body": body,
        },
        "data": {
            "event": event.event.as_str(),
            "ac_connected": event.ac_connected.to_string(),
            "battery_percentage": event
                .battery_percentage
                .map(|p| p.round().to_string())
                .unwrap_or_default(),
        }
    });
    let resp = client
        .post("https://fcm.googleapis.com/fcm/send")
        .header("Authorization", format!("key={server_key}"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .context("FCM legacy send")?
        .error_for_status()
        .context("FCM legacy HTTP error")?;
    let body: serde_json::Value = resp.json().unwrap_or_default();
    if body.get("failure").and_then(|v| v.as_u64()) == Some(1) {
        bail!("FCM legacy failure: {body}");
    }
    Ok(())
}

fn send_v1(
    project_id: &str,
    access_token: &str,
    token: &str,
    title: &str,
    body: &str,
    event: &PowerEvent,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;
    let url = format!("https://fcm.googleapis.com/v1/projects/{project_id}/messages:send");
    let payload = serde_json::json!({
        "message": {
            "token": token,
            "notification": {
                "title": title,
                "body": body,
            },
            "data": {
                "event": event.event.as_str(),
                "ac_connected": event.ac_connected.to_string(),
                "battery_percentage": event
                    .battery_percentage
                    .map(|p| p.round().to_string())
                    .unwrap_or_default(),
            },
            "android": { "priority": "HIGH" },
        }
    });
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .context("FCM v1 send")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        bail!("FCM v1 HTTP {status}: {text}");
    }
    Ok(())
}

pub fn load_tokens_file(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)
        .with_context(|| format!("reading FCM tokens {}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect())
}

pub fn save_tokens_file(path: &Path, tokens: &[String]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = String::from("# FCM device tokens (one per line)\n");
    for t in tokens {
        out.push_str(t);
        out.push('\n');
    }
    fs::write(path, out).with_context(|| format!("writing FCM tokens {}", path.display()))?;
    Ok(())
}

pub fn default_tokens_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("power-monitor")
        .join("fcm_tokens.txt")
}

fn truncate(token: &str) -> String {
    if token.len() <= 12 {
        token.to_string()
    } else {
        format!("{}…", &token[..12])
    }
}
