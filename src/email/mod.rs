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
    /// Verified sender address in Brevo (never the *@smtp-brevo.com login).
    pub from_address: String,
    pub from_name: String,
    pub to: String,
}

impl SmtpSettings {
    /// Returns `Ok(None)` when email is intentionally unconfigured.
    pub fn from_env() -> Result<Option<Self>> {
        let host = first_env(&["SMTP_HOST", "MAIL_HOST"])
            .unwrap_or_else(|| "smtp-relay.brevo.com".into());
        let user = first_env(&["SMTP_USER", "MAIL_USERNAME"]);
        let password = first_env(&["SMTP_PASSWORD", "SMTP_PASS", "MAIL_PASSWORD"]);

        // Prefer explicit address (Laravel-style). SMTP_FROM may be "Name <email>".
        let from_address = first_env(&["SMTP_FROM_ADDRESS", "MAIL_FROM_ADDRESS", "SMTP_FROM"])
            .map(|raw| extract_email(&raw))
            .filter(|s| !s.is_empty());

        let from_name = first_env(&["SMTP_FROM_NAME", "MAIL_FROM_NAME"])
            .or_else(|| first_env(&["SMTP_FROM"]).and_then(|raw| extract_display_name(&raw)))
            .unwrap_or_else(|| "Power Monitor".into());

        let any = user.is_some() || password.is_some() || from_address.is_some();
        let all = user.is_some() && password.is_some() && from_address.is_some();
        if any && !all {
            return Err(anyhow!(
                "incomplete SMTP config: need SMTP_USER (or MAIL_USERNAME), \
                 SMTP_PASSWORD (or MAIL_PASSWORD), and SMTP_FROM_ADDRESS \
                 (or MAIL_FROM_ADDRESS). From must be a sender verified in Brevo — \
                 not your *@smtp-brevo.com login."
            ));
        }
        if !all {
            return Ok(None);
        }

        let from_address = from_address.unwrap();
        if from_address.ends_with("@smtp-brevo.com") {
            return Err(anyhow!(
                "SMTP_FROM_ADDRESS must be your verified sender email in Brevo, \
                 not the SMTP login ({from_address})"
            ));
        }

        let port = first_env(&["SMTP_PORT", "MAIL_PORT"])
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let to = first_env(&["SMTP_TO", "MAIL_TO"])
            .unwrap_or_else(|| "katoemmy001@gmail.com".into());

        Ok(Some(Self {
            host,
            port,
            user: user.unwrap(),
            password: password.unwrap(),
            from_address,
            from_name,
            to,
        }))
    }
}

pub fn maybe_send(cfg: &EmailConfig, smtp: &SmtpSettings, event: &PowerEvent) {
    maybe_send_to(cfg, smtp, event, &[smtp.to.clone()]);
}

/// Send to explicit recipient list (seeded users). Falls back to `smtp.to` if empty.
pub fn maybe_send_to(
    cfg: &EmailConfig,
    smtp: &SmtpSettings,
    event: &PowerEvent,
    recipients: &[String],
) {
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

    let targets: Vec<&str> = if recipients.is_empty() {
        vec![smtp.to.as_str()]
    } else {
        recipients.iter().map(String::as_str).collect()
    };

    for to in targets {
        if let Err(err) = send_to(smtp, to, &subject, &body) {
            warn!(error = %err, to = %to, "failed to send email notification");
        } else {
            info!(
                to = %to,
                from = %smtp.from_address,
                subject = %subject,
                "email notification sent"
            );
        }
    }
}

fn send_to(smtp: &SmtpSettings, to_addr: &str, subject: &str, body: &str) -> Result<()> {
    let from = Mailbox::new(
        Some(smtp.from_name.clone()),
        smtp.from_address
            .parse()
            .with_context(|| format!("invalid SMTP_FROM_ADDRESS: {}", smtp.from_address))?,
    );
    let to = Mailbox::new(
        None,
        to_addr
            .parse()
            .with_context(|| format!("invalid recipient address: {to_addr}"))?,
    );

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

    let response = mailer.send(&email).context("SMTP send")?;
    if !response.is_positive() {
        return Err(anyhow!("SMTP rejected message: {response:?}"));
    }
    Ok(())
}

fn first_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| {
        env::var(k).ok().and_then(|v| {
            let v = v.trim().trim_matches('"').trim_matches('\'').trim();
            if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            }
        })
    })
}

/// Pull `user@host` from `Name <user@host>` or bare address.
fn extract_email(raw: &str) -> String {
    let raw = raw.trim().trim_matches('"').trim_matches('\'');
    if let (Some(start), Some(end)) = (raw.find('<'), raw.find('>')) {
        if end > start {
            return raw[start + 1..end].trim().to_string();
        }
    }
    raw.to_string()
}

fn extract_display_name(raw: &str) -> Option<String> {
    let raw = raw.trim().trim_matches('"').trim_matches('\'');
    if let Some(start) = raw.find('<') {
        let name = raw[..start].trim().trim_matches('"').trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}
