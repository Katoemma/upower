use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryState {
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    NotCharging,
}

impl BatteryState {
    pub fn from_upower(state: u32) -> Self {
        match state {
            1 => Self::Charging,
            2 => Self::Discharging,
            3 => Self::Empty,
            4 => Self::FullyCharged,
            5 | 6 => Self::NotCharging,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Charging => "charging",
            Self::Discharging => "discharging",
            Self::Empty => "empty",
            Self::FullyCharged => "fully_charged",
            Self::NotCharging => "not_charging",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub id: String,
    pub percentage: Option<f64>,
    pub state: BatteryState,
    pub health: Option<f64>,
    pub energy_now_wh: Option<f64>,
    pub energy_full_wh: Option<f64>,
    pub energy_full_design_wh: Option<f64>,
    pub temperature_celsius: Option<f64>,
    pub time_to_full_seconds: Option<i64>,
    pub time_remaining_seconds: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerState {
    pub ac_connected: bool,
    pub batteries: Vec<BatteryInfo>,
}

impl PowerState {
    /// Primary battery: DisplayDevice aggregate if present, else first battery.
    pub fn primary_battery(&self) -> Option<&BatteryInfo> {
        self.batteries
            .iter()
            .find(|b| b.id == "DisplayDevice")
            .or_else(|| self.batteries.first())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    AcConnected,
    AcDisconnected,
    ChargingStarted,
    ChargingStopped,
    BatteryFull,
    BatteryPercentageChanged,
    LowBattery,
    CriticalBattery,
}

impl EventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AcConnected => "ac_connected",
            Self::AcDisconnected => "ac_disconnected",
            Self::ChargingStarted => "charging_started",
            Self::ChargingStopped => "charging_stopped",
            Self::BatteryFull => "battery_full",
            Self::BatteryPercentageChanged => "battery_percentage_changed",
            Self::LowBattery => "low_battery",
            Self::CriticalBattery => "critical_battery",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ac_connected" => Some(Self::AcConnected),
            "ac_disconnected" => Some(Self::AcDisconnected),
            "charging_started" => Some(Self::ChargingStarted),
            "charging_stopped" => Some(Self::ChargingStopped),
            "battery_full" => Some(Self::BatteryFull),
            "battery_percentage_changed" => Some(Self::BatteryPercentageChanged),
            "low_battery" => Some(Self::LowBattery),
            "critical_battery" => Some(Self::CriticalBattery),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerEvent {
    pub event: EventType,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub battery_percentage: Option<f64>,
    pub battery_state: Option<BatteryState>,
    pub ac_connected: bool,
}

mod monitor;

pub use monitor::PowerMonitor;
