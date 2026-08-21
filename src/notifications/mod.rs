use notify_rust::Notification;
use tracing::warn;

use crate::config::{EmailConfig, NotificationConfig, PushConfig};
use crate::email::SmtpSettings;
use crate::power::{EventType, PowerEvent};
use crate::push::FcmClient;

/// Desktop + optional email + optional FCM (safe to call from `spawn_blocking`).
pub fn dispatch(
    desktop: &NotificationConfig,
    email_cfg: &EmailConfig,
    smtp: Option<&SmtpSettings>,
    email_recipients: &[String],
    push_cfg: &PushConfig,
    fcm: Option<&FcmClient>,
    push_tokens: &[String],
    event: &PowerEvent,
) {
    maybe_desktop(desktop, event);
    if let Some(smtp) = smtp {
        crate::email::maybe_send_to(email_cfg, smtp, event, email_recipients);
    }
    if let Some(fcm) = fcm {
        if push_tokens.is_empty() {
            fcm.maybe_send(push_cfg, event);
        } else {
            fcm.maybe_send_to(push_cfg, event, push_tokens);
        }
    }
}

pub fn maybe_desktop(cfg: &NotificationConfig, event: &PowerEvent) {
    if !cfg.enabled {
        return;
    }

    let Some((title, body)) = message_for_event(
        event,
        cfg.ac_connected,
        cfg.ac_disconnected,
        cfg.fully_charged,
        cfg.low_battery,
        cfg.critical_battery,
    ) else {
        return;
    };

    if let Err(err) = Notification::new()
        .summary(&title)
        .body(&body)
        .appname("power-monitor")
        .timeout(5000)
        .show()
    {
        warn!(error = %err, "failed to show notification");
    }
}

pub fn message_for_event(
    event: &PowerEvent,
    ac_connected: bool,
    ac_disconnected: bool,
    fully_charged: bool,
    low_battery: bool,
    critical_battery: bool,
) -> Option<(String, String)> {
    match event.event {
        EventType::AcDisconnected if ac_disconnected => Some((
            "Power disconnected".into(),
            format!(
                "Your computer is now running on battery.{}",
                pct_suffix(event.battery_percentage)
            ),
        )),
        EventType::AcConnected if ac_connected => Some((
            "Power connected".into(),
            format!(
                "Your computer is charging.{}",
                pct_suffix(event.battery_percentage)
            ),
        )),
        EventType::LowBattery if low_battery => Some((
            "Low battery".into(),
            format!(
                "Battery level has reached {}%.",
                event
                    .battery_percentage
                    .map(|p| p.round() as u8)
                    .unwrap_or(0)
            ),
        )),
        EventType::CriticalBattery if critical_battery => Some((
            "Critical battery".into(),
            format!(
                "Battery level has reached {}%.",
                event
                    .battery_percentage
                    .map(|p| p.round() as u8)
                    .unwrap_or(0)
            ),
        )),
        EventType::BatteryFull if fully_charged => Some((
            "Battery fully charged".into(),
            "Battery is at 100%.".into(),
        )),
        _ => None,
    }
}

fn pct_suffix(pct: Option<f64>) -> String {
    match pct {
        Some(p) => format!(" Battery: {}%", p.round() as u8),
        None => String::new(),
    }
}
