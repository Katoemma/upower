use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::power::{BatteryState, EventType, PowerEvent};

pub struct EventStore {
    pool: SqlitePool,
}

impl EventStore {
    pub async fn open(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .with_context(|| format!("creating data dir {}", data_dir.display()))?;
        let db_path = data_dir.join("power-monitor.db");
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .with_context(|| format!("opening sqlite {}", db_path.display()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS power_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                battery_percentage REAL,
                battery_state TEXT,
                ac_connected INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("migrating power_events table")?;

        Ok(Self { pool })
    }

    pub async fn insert(&self, event: &PowerEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO power_events
                (event_type, timestamp, battery_percentage, battery_state, ac_connected)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.event.as_str())
        .bind(event.timestamp.to_rfc3339())
        .bind(event.battery_percentage)
        .bind(event.battery_state.map(|s| s.as_str()))
        .bind(event.ac_connected as i64)
        .execute(&self.pool)
        .await
        .context("inserting power event")?;
        Ok(())
    }

    pub async fn query(
        &self,
        page: u32,
        limit: u32,
        event_type: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<PowerEvent>> {
        let limit = limit.clamp(1, 500);
        let page = page.max(1);
        let offset = (page - 1) * limit;

        let mut sql = String::from(
            "SELECT event_type, timestamp, battery_percentage, battery_state, ac_connected \
             FROM power_events WHERE 1=1",
        );
        if event_type.is_some() {
            sql.push_str(" AND event_type = ?");
        }
        if from.is_some() {
            sql.push_str(" AND timestamp >= ?");
        }
        if to.is_some() {
            sql.push_str(" AND timestamp <= ?");
        }
        sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query(&sql);
        if let Some(t) = event_type {
            q = q.bind(t);
        }
        if let Some(f) = from {
            q = q.bind(f);
        }
        if let Some(t) = to {
            q = q.bind(t);
        }
        q = q.bind(limit as i64).bind(offset as i64);

        let rows = q.fetch_all(&self.pool).await.context("querying events")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let event_type: String = row.get("event_type");
            let timestamp: String = row.get("timestamp");
            let battery_percentage: Option<f64> = row.get("battery_percentage");
            let battery_state: Option<String> = row.get("battery_state");
            let ac_connected: i64 = row.get("ac_connected");

            let Some(event) = EventType::parse(&event_type) else {
                continue;
            };
            let ts = DateTime::parse_from_rfc3339(&timestamp)
                .map(|dt| dt.with_timezone(&Local))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(&timestamp, "%Y-%m-%d %H:%M:%S")
                        .map(|naive| naive.and_local_timezone(Local).unwrap())
                })
                .unwrap_or_else(|_| Local::now());

            let battery_state = battery_state.as_deref().and_then(|s| match s {
                "unknown" => Some(BatteryState::Unknown),
                "charging" => Some(BatteryState::Charging),
                "discharging" => Some(BatteryState::Discharging),
                "empty" => Some(BatteryState::Empty),
                "fully_charged" => Some(BatteryState::FullyCharged),
                "not_charging" => Some(BatteryState::NotCharging),
                _ => None,
            });

            out.push(PowerEvent {
                event,
                timestamp: ts,
                battery_percentage,
                battery_state,
                ac_connected: ac_connected != 0,
            });
        }
        Ok(out)
    }
}
