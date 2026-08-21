use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use tracing::debug;
use zbus::fdo::PropertiesProxy;
use zbus::proxy;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

use crate::power::{BatteryInfo, BatteryState, PowerState};

const UPOWER_DEST: &str = "org.freedesktop.UPower";
const UPOWER_PATH: &str = "/org/freedesktop/UPower";

#[proxy(
    default_service = "org.freedesktop.UPower",
    interface = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    #[zbus(property)]
    fn on_battery(&self) -> zbus::Result<bool>;

    fn enumerate_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn get_display_device(&self) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    default_service = "org.freedesktop.UPower",
    interface = "org.freedesktop.UPower.Device"
)]
trait Device {
    #[zbus(property)]
    fn native_path(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn type_(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn power_supply(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn online(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn energy(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn energy_full(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn energy_full_design(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn capacity(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn temperature(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn time_to_empty(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn time_to_full(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn is_present(&self) -> zbus::Result<bool>;
}

const TYPE_LINE_POWER: u32 = 1;
const TYPE_BATTERY: u32 = 2;

pub struct UPowerClient {
    conn: Connection,
    upower: UPowerProxy<'static>,
}

impl UPowerClient {
    pub async fn connect() -> Result<Self> {
        let conn = Connection::system()
            .await
            .context("connecting to system D-Bus")?;
        let upower = UPowerProxy::new(&conn)
            .await
            .context("creating UPower proxy")?;
        Ok(Self { conn, upower })
    }

    pub async fn read_snapshot(&self) -> Result<PowerState> {
        let on_battery = self.upower.on_battery().await.unwrap_or(false);
        let mut ac_connected = !on_battery;
        let mut batteries = Vec::new();

        if let Ok(path) = self.upower.get_display_device().await {
            if let Ok(Some(info)) = self.read_battery_at(&path, "DisplayDevice").await {
                batteries.push(info);
            }
        }

        let devices = self.upower.enumerate_devices().await.unwrap_or_default();
        for path in devices {
            let device = DeviceProxy::builder(&self.conn)
                .path(path.clone())
                .context("device path")?
                .build()
                .await
                .context("device proxy")?;

            let dtype = device.type_().await.unwrap_or(0);
            if dtype != TYPE_BATTERY {
                continue;
            }
            if !device.is_present().await.unwrap_or(true) {
                continue;
            }

            let id = device
                .native_path()
                .await
                .unwrap_or_else(|_| path.to_string());

            if batteries.iter().any(|b| b.id == id) {
                continue;
            }

            if let Ok(Some(info)) = self.read_battery_at(&path, &id).await {
                batteries.push(info);
            }
        }

        if batteries.is_empty() {
            ac_connected = self.any_line_power_online().await.unwrap_or(ac_connected);
        }

        // DisplayDevice often omits Capacity; copy fields from a physical battery when missing.
        let donor = batteries
            .iter()
            .find(|b| b.id != "DisplayDevice")
            .cloned();
        if let Some(donor) = donor {
            if let Some(display) = batteries.iter_mut().find(|b| b.id == "DisplayDevice") {
                if display.health.is_none() {
                    display.health = donor.health;
                }
                if display.energy_full_design_wh.is_none() {
                    display.energy_full_design_wh = donor.energy_full_design_wh;
                }
                if display.temperature_celsius.is_none() {
                    display.temperature_celsius = donor.temperature_celsius;
                }
            }
        }

        Ok(PowerState {
            ac_connected,
            batteries,
        })
    }

    async fn any_line_power_online(&self) -> Result<bool> {
        let devices = self.upower.enumerate_devices().await?;
        for path in devices {
            let device = DeviceProxy::builder(&self.conn)
                .path(path)?
                .build()
                .await?;
            if device.type_().await.unwrap_or(0) == TYPE_LINE_POWER
                && device.online().await.unwrap_or(false)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn read_battery_at(
        &self,
        path: &OwnedObjectPath,
        id: &str,
    ) -> Result<Option<BatteryInfo>> {
        let device = DeviceProxy::builder(&self.conn)
            .path(path.clone())?
            .build()
            .await?;

        let dtype = device.type_().await.unwrap_or(0);
        if id != "DisplayDevice" && dtype != TYPE_BATTERY {
            return Ok(None);
        }

        let percentage = device.percentage().await.ok().filter(|p| *p >= 0.0);
        let state = BatteryState::from_upower(device.state().await.unwrap_or(0));
        let energy_now_wh = positive_f64(device.energy().await.ok());
        let energy_full_wh = positive_f64(device.energy_full().await.ok());
        let energy_full_design_wh = positive_f64(device.energy_full_design().await.ok());
        let health = positive_f64(device.capacity().await.ok());
        let temperature_celsius = device.temperature().await.ok().filter(|t| *t > 0.0);
        let time_to_empty = device.time_to_empty().await.ok().filter(|t| *t > 0);
        let time_to_full = device.time_to_full().await.ok().filter(|t| *t > 0);

        if id == "DisplayDevice"
            && percentage.is_none()
            && energy_now_wh.is_none()
            && matches!(state, BatteryState::Unknown)
        {
            return Ok(None);
        }

        Ok(Some(BatteryInfo {
            id: id.to_string(),
            percentage,
            state,
            health,
            energy_now_wh,
            energy_full_wh,
            energy_full_design_wh,
            temperature_celsius,
            time_to_full_seconds: time_to_full,
            time_remaining_seconds: time_to_empty,
        }))
    }

    /// Wait for a PropertiesChanged on UPower or DisplayDevice, or poll on timeout.
    pub async fn wait_for_change(&self, timeout: Duration) -> Result<()> {
        let upower_props = PropertiesProxy::builder(&self.conn)
            .destination(UPOWER_DEST)?
            .path(UPOWER_PATH)?
            .build()
            .await?;
        let mut upower_changes = upower_props.receive_properties_changed().await?;

        let display_path = self.upower.get_display_device().await.ok();
        let mut display_changes = if let Some(path) = display_path {
            let props = PropertiesProxy::builder(&self.conn)
                .destination(UPOWER_DEST)?
                .path(path)?
                .build()
                .await?;
            Some(props.receive_properties_changed().await?)
        } else {
            None
        };

        tokio::select! {
            msg = upower_changes.next() => {
                if msg.is_some() {
                    debug!("UPower properties changed");
                    Ok(())
                } else {
                    Err(anyhow!("UPower properties stream ended"))
                }
            }
            msg = async {
                if let Some(ref mut s) = display_changes {
                    s.next().await
                } else {
                    std::future::pending().await
                }
            } => {
                if msg.is_some() {
                    debug!("DisplayDevice properties changed");
                    Ok(())
                } else {
                    Err(anyhow!("DisplayDevice stream ended"))
                }
            }
            _ = tokio::time::sleep(timeout) => {
                debug!("wait timeout; refreshing snapshot");
                Ok(())
            }
        }
    }
}

fn positive_f64(v: Option<f64>) -> Option<f64> {
    v.filter(|x| *x > 0.0)
}
