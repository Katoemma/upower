//! Seeded users (no public registration).

use anyhow::{bail, Result};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub notify_email: bool,
}

pub fn normalize_email(email: &str) -> Result<String> {
    let email = email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') || email.len() > 254 {
        bail!("invalid email address");
    }
    Ok(email)
}

pub fn hash_password(password: &str) -> Result<String> {
    if password.len() < 6 {
        bail!("password must be at least 6 characters");
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash password: {e}"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|e| anyhow::anyhow!("invalid password hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
