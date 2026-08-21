//! JWT auth for the localhost API.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::users::User;

const TOKEN_TTL_SECS: u64 = 60 * 60 * 24 * 30; // 30 days

#[derive(Debug, Clone)]
pub struct AuthState {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
    /// When true, protected routes require a valid Bearer token.
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    email: String,
    exp: u64,
    iat: u64,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub email: String,
}

impl AuthState {
    pub fn load(data_dir: &Path, user_count: i64) -> Result<Self> {
        let secret = load_or_create_secret(data_dir)?;
        Ok(Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            required: user_count > 0,
        })
    }

    pub fn issue_token(&self, user: &User) -> Result<String> {
        let now = now_secs();
        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            iat: now,
            exp: now + TOKEN_TTL_SECS,
        };
        encode(&Header::default(), &claims, &self.encoding).context("encoding JWT")
    }

    pub fn verify_token(&self, token: &str) -> Result<AuthUser> {
        let data = decode::<Claims>(token, &self.decoding, &Validation::default())
            .map_err(|e| anyhow!("invalid token: {e}"))?;
        Ok(AuthUser {
            id: data.claims.sub,
            email: data.claims.email,
        })
    }
}

fn load_or_create_secret(data_dir: &Path) -> Result<String> {
    if let Ok(s) = std::env::var("AUTH_JWT_SECRET") {
        let s = s.trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    fs::create_dir_all(data_dir)?;
    let path = data_dir.join("jwt_secret");
    if path.exists() {
        return Ok(fs::read_to_string(&path)?.trim().to_string());
    }
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let secret = hex::encode(bytes);
    // Fallback without hex crate: use base64-ish via format
    let secret = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    fs::write(&path, &secret).with_context(|| format!("writing {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(secret)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ").or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim().to_string())
}

/// Extractor: requires auth when users exist.
pub struct OptionalAuth(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Some(token) = extract_bearer(&parts.headers).or_else(|| {
            // Also allow ?token= for WebSocket-friendly clients
            parts
                .uri
                .query()
                .and_then(|q| {
                    q.split('&').find_map(|pair| {
                        let mut it = pair.splitn(2, '=');
                        let k = it.next()?;
                        let v = it.next()?;
                        if k == "token" {
                            Some(urlencoding_decode(v))
                        } else {
                            None
                        }
                    })
                })
        }) else {
            if state.auth.required {
                return Err((StatusCode::UNAUTHORIZED, "missing Bearer token".into()));
            }
            return Ok(OptionalAuth(None));
        };

        match state.auth.verify_token(&token) {
            Ok(user) => Ok(OptionalAuth(Some(user))),
            Err(err) => {
                if state.auth.required {
                    Err((StatusCode::UNAUTHORIZED, err.to_string()))
                } else {
                    Ok(OptionalAuth(None))
                }
            }
        }
    }
}

fn urlencoding_decode(s: &str) -> String {
    // Minimal decode for JWT (usually unreserved).
    s.replace("%2B", "+")
        .replace("%2F", "/")
        .replace("%3D", "=")
}

/// Convenience: ensure auth when required; returns user when present.
pub async fn require_user(
    auth: OptionalAuth,
    required: bool,
) -> Result<Option<AuthUser>, (StatusCode, String)> {
    if required && auth.0.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "authentication required".into()));
    }
    Ok(auth.0)
}

pub fn token_file_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("power-monitor")
        .join("token")
}

pub fn save_cli_token(token: &str) -> Result<()> {
    let path = token_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, token.trim())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn load_cli_token() -> Option<String> {
    std::env::var("POWER_MONITOR_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| fs::read_to_string(token_file_path()).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[allow(dead_code)]
pub type SharedAuth = Arc<AuthState>;
