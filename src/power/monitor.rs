use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use crate::config::{BatteryConfig, Config, EmailConfig, NotificationConfig, PushConfig};
use crate::database::EventStore;
use crate::email::SmtpSettings;
use crate::notifications;
use crate::power::{BatteryState, EventType, PowerEvent, PowerState};
use crate::push::FcmClient;
use crate::upower::UPowerClient;

pub struct PowerMonitor {
    client: UPowerClient,
    state: Arc<RwLock<PowerState>>,
    event_tx: broadcast::Sender<PowerEvent>,
    store: Arc<EventStore>,
    #[allow(dead_code)]
    config: Config,
    notif: NotificationConfig,
    email: EmailConfig,
    smtp: Option<SmtpSettings>,
    push: PushConfig,
    fcm: Option<FcmClient>,
    battery: BatteryConfig,
    prev: PowerState,
    /// Track whether we already fired low/critical for this discharge cycle.
    fired_low: bool,
    fired_critical: bool,
}

impl PowerMonitor {
    pub fn new(
        client: UPowerClient,
        state: Arc<RwLock<PowerState>>,
        event_tx: broadcast::Sender<PowerEvent>,
        store: Arc<EventStore>,
        config: Config,
        notif: NotificationConfig,
        email: EmailConfig,
        smtp: Option<SmtpSettings>,
        push: PushConfig,
        fcm: Option<FcmClient>,
        battery: BatteryConfig,
    ) -> Self {
        Self {
            client,
            state,
            event_tx,
            store,
            config,
            notif,
            email,
            smtp,
            push,
            fcm,
            battery,
            prev: PowerState::default(),
            fired_low: false,
            fired_critical: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Seed previous from current shared state.
        self.prev = self.state.read().await.clone();
        self.seed_threshold_flags();

        loop {
            match self.client.wait_for_change(Duration::from_secs(30)).await {
                Ok(()) => {}
                Err(err) => {
                    warn!(error = %err, "UPower wait failed; reconnecting");
                    self.reconnect().await;
                    continue;
                }
            }

            match self.client.read_snapshot().await {
                Ok(snapshot) => self.apply_snapshot(snapshot).await,
                Err(err) => {
                    warn!(error = %err, "failed to read UPower snapshot");
                    self.reconnect().await;
                }
            }
        }
    }

    fn seed_threshold_flags(&mut self) {
        if let Some(pct) = self
            .prev
            .primary_battery()
            .and_then(|b| b.percentage)
            .map(|p| p as u8)
        {
            if pct <= self.battery.critical_threshold {
                self.fired_critical = true;
                self.fired_low = true;
            } else if pct <= self.battery.low_threshold {
                self.fired_low = true;
            }
        }
    }

    async fn reconnect(&mut self) {
        let mut delay = Duration::from_secs(1);
        loop {
            tokio::time::sleep(delay).await;
            match UPowerClient::connect().await {
                Ok(client) => {
                    info!("reconnected to UPower");
                    self.client = client;
                    if let Ok(snapshot) = self.client.read_snapshot().await {
                        self.apply_snapshot(snapshot).await;
                    }
                    return;
                }
                Err(err) => {
                    warn!(error = %err, "UPower reconnect failed");
                    delay = (delay * 2).min(Duration::from_secs(30));
                }
            }
        }
    }

    async fn apply_snapshot(&mut self, snapshot: PowerState) {
        let prev = self.prev.clone();
        let events = self.diff(&prev, &snapshot);
        {
            let mut guard = self.state.write().await;
            *guard = snapshot.clone();
        }

        for event in events {
            info!(
                event = event.event.as_str(),
                battery = ?event.battery_percentage,
                ac = event.ac_connected,
                "power event"
            );

            if let Err(err) = self.store.insert(&event).await {
                warn!(error = %err, "failed to persist event");
            }

            let _ = self.event_tx.send(event.clone());

            let email_recipients = self
                .store
                .notification_emails()
                .await
                .unwrap_or_default();
            let mut push_tokens = self.store.list_all_fcm_tokens().await.unwrap_or_default();
            // Also include legacy file tokens.
            if let Some(fcm) = &self.fcm {
                if let Ok(file_tokens) = fcm.load_tokens() {
                    for t in file_tokens {
                        if !push_tokens.iter().any(|x| x == &t) {
                            push_tokens.push(t);
                        }
                    }
                }
            }

            let notif = self.notif.clone();
            let email_cfg = self.email.clone();
            let smtp = self.smtp.clone();
            let push_cfg = self.push.clone();
            let fcm = self.fcm.clone();
            let ev = event.clone();
            tokio::task::spawn_blocking(move || {
                notifications::dispatch(
                    &notif,
                    &email_cfg,
                    smtp.as_ref(),
                    &email_recipients,
                    &push_cfg,
                    fcm.as_ref(),
                    &push_tokens,
                    &ev,
                );
            });
        }

        self.prev = snapshot;
    }

    fn diff(&mut self, prev: &PowerState, next: &PowerState) -> Vec<PowerEvent> {
        let mut events = Vec::new();
        let now = chrono::Local::now();
        let pct = next.primary_battery().and_then(|b| b.percentage);
        let bat_state = next.primary_battery().map(|b| b.state);

        if prev.ac_connected != next.ac_connected {
            let event_type = if next.ac_connected {
                EventType::AcConnected
            } else {
                EventType::AcDisconnected
            };
            events.push(PowerEvent {
                event: event_type,
                timestamp: now,
                battery_percentage: pct,
                battery_state: bat_state,
                ac_connected: next.ac_connected,
            });
        }

        let prev_state = prev.primary_battery().map(|b| b.state);
        let next_state = bat_state;

        if prev_state != next_state {
            match next_state {
                Some(BatteryState::Charging)
                    if prev_state != Some(BatteryState::Charging) =>
                {
                    events.push(PowerEvent {
                        event: EventType::ChargingStarted,
                        timestamp: now,
                        battery_percentage: pct,
                        battery_state: bat_state,
                        ac_connected: next.ac_connected,
                    });
                    // Reset threshold flags when charging resumes / AC returns.
                    self.fired_low = false;
                    self.fired_critical = false;
                }
                Some(BatteryState::Discharging)
                    if prev_state == Some(BatteryState::Charging)
                        || prev_state == Some(BatteryState::FullyCharged) =>
                {
                    events.push(PowerEvent {
                        event: EventType::ChargingStopped,
                        timestamp: now,
                        battery_percentage: pct,
                        battery_state: bat_state,
                        ac_connected: next.ac_connected,
                    });
                }
                Some(BatteryState::FullyCharged) => {
                    events.push(PowerEvent {
                        event: EventType::BatteryFull,
                        timestamp: now,
                        battery_percentage: pct,
                        battery_state: bat_state,
                        ac_connected: next.ac_connected,
                    });
                }
                _ => {}
            }
        }

        let prev_pct = prev.primary_battery().and_then(|b| b.percentage);
        if let (Some(p), Some(n)) = (prev_pct, pct) {
            // Integer percentage change only (avoid noise from float jitter).
            if p.round() as i64 != n.round() as i64 {
                events.push(PowerEvent {
                    event: EventType::BatteryPercentageChanged,
                    timestamp: now,
                    battery_percentage: pct,
                    battery_state: bat_state,
                    ac_connected: next.ac_connected,
                });
            }
        }

        if let Some(n) = pct.map(|p| p as u8) {
            // Reset threshold flags when battery recovers above thresholds.
            if n > self.battery.low_threshold {
                self.fired_low = false;
            }
            if n > self.battery.critical_threshold {
                self.fired_critical = false;
            }

            if n <= self.battery.critical_threshold && !self.fired_critical {
                self.fired_critical = true;
                self.fired_low = true;
                events.push(PowerEvent {
                    event: EventType::CriticalBattery,
                    timestamp: now,
                    battery_percentage: pct,
                    battery_state: bat_state,
                    ac_connected: next.ac_connected,
                });
            } else if n <= self.battery.low_threshold && !self.fired_low {
                self.fired_low = true;
                events.push(PowerEvent {
                    event: EventType::LowBattery,
                    timestamp: now,
                    battery_percentage: pct,
                    battery_state: bat_state,
                    ac_connected: next.ac_connected,
                });
            }
        }

        events
    }
}
