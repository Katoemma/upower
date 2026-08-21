use std::env;

use anyhow::{anyhow, Context, Result};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use tracing::{info, warn};

use crate::config::EmailConfig;
use crate::power::PowerEvent;

/// Brevo (or any) SMTP settings loaded from environment / `.env`.
#[derive(Debug, Clone)]
pub struct SmtpSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub from: String,
    pub to: String,
}

impl SmtpSettings {
    /// Returns `Ok(None)` when email is intentionally unconfigured.
    pub fn from_env() -> Result<Option<Self>> {
        let host = env::var("SMTP_HOST").ok().filter(|s| !s.is_empty());
        let user = env::var("SMTP_USER").ok().filter(|s| !s.is_empty());
        let password = env::var("SMTP_PASSWORD")
            .or_else(|_| env::var("SMTP_PASS"))
            .ok()
            .filter(|s| !s.is_empty());
        let from = env::var("SMTP_FROM").ok().filter(|s| !s.is_empty());

        // Partial config is an error so misconfiguration is obvious.
        let any = host.is_some() || user.is_some() || password.is_some() || from.is_some();
        let all = host.is_some() && user.is_some() && password.is_some() && from.is_some();
        if any && !all {
            return Err(anyhow!(
                "incomplete SMTP config: set SMTP_HOST, SMTP_USER, SMTP_PASSWORD, and SMTP_FROM"
            ));
        }
        if !all {
            return Ok(None);
        }

        let port = env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let to = env::var("SMTP_TO").unwrap_or_else(|_| "nativesenior@gmail.com".into());

        Ok(Some(Self {
            host: host.unwrap(),
            port,
            user: user.unwrap(),
            password: password.unwrap(),
            from: from.unwrap(),
            to,
        }))
    }
}

pub fn maybe_send(cfg: &EmailConfig, smtp: &SmtpSettings, event: &PowerEvent) {
    if !cfg.enabled {
        return;
    }
    let Some((subject, body)) = crate::notifications::message_for_event(
        event,
        cfg.ac_connected,
        cfg.ac_disconnected,
        cfg.fully_charged,
        cfg.low_battery,
        cfg.critical_battery,
    ) else {
        return;
    };

    if let Err(err) = send(smtp, &subject, &body) {
        warn!(error = %err, "failed to send email notification");
    } else {
        info!(to = %smtp.to, subject = %subject, "email notification sent");
    }
}

fn send(smtp: &SmtpSettings, subject: &str, body: &str) -> Result<()> {
    let from: Mailbox = smtp
        .from
        .parse()
        .with_context(|| format!("invalid SMTP_FROM address: {}", smtp.from))?;
    let to: Mailbox = smtp
        .to
        .parse()
        .with_context(|| format!("invalid SMTP_TO address: {}", smtp.to))?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(format!("[power-monitor] {subject}"))
        .body(body.to_string())
        .context("building email")?;

    let creds = Credentials::new(smtp.user.clone(), smtp.password.clone());
    let mailer = SmtpTransport::starttls_relay(&smtp.host)
        .with_context(|| format!("SMTP relay {}", smtp.host))?
        .port(smtp.port)
        .credentials(creds)
        .build();

    mailer.send(&email).context("SMTP send")?;
    Ok(())
}
