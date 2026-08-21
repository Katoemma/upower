use notify_rust::Notification;
use tracing::warn;

use crate::config::NotificationConfig;
use crate::power::{EventType, PowerEvent};

pub fn maybe_notify(cfg: &NotificationConfig, event: &PowerEvent) {
    if !cfg.enabled {
        return;
    }

    let (title, body) = match event.event {
        EventType::AcDisconnected if cfg.ac_disconnected => (
            "Power disconnected",
            format!(
                "Your computer is now running on battery.{}",
                pct_suffix(event.battery_percentage)
            ),
        ),
        EventType::AcConnected if cfg.ac_connected => (
            "Power connected",
            format!(
                "Your computer is charging.{}",
                pct_suffix(event.battery_percentage)
            ),
        ),
        EventType::LowBattery if cfg.low_battery => (
            "Low battery",
            format!(
                "Battery level has reached {}%.",
                event
                    .battery_percentage
                    .map(|p| p.round() as u8)
                    .unwrap_or(0)
            ),
        ),
        EventType::CriticalBattery if cfg.critical_battery => (
            "Critical battery",
            format!(
                "Battery level has reached {}%.",
                event
                    .battery_percentage
                    .map(|p| p.round() as u8)
                    .unwrap_or(0)
            ),
        ),
        EventType::BatteryFull if cfg.fully_charged => (
            "Battery fully charged",
            "Battery is at 100%.".to_string(),
        ),
        _ => return,
    };

    if let Err(err) = Notification::new()
        .summary(title)
        .body(&body)
        .appname("power-monitor")
        .timeout(5000)
        .show()
    {
        warn!(error = %err, "failed to show notification");
    }
}

fn pct_suffix(pct: Option<f64>) -> String {
    match pct {
        Some(p) => format!(" Battery: {}%", p.round() as u8),
        None => String::new(),
    }
}
