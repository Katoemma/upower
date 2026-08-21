//! SQLite persistence for events, users, and per-user FCM tokens.

use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, NaiveDateTime};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::power::{BatteryState, EventType, PowerEvent};
use crate::users::{self, User};

#[derive(Clone)]
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

        migrate(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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

    // --- users ---

    pub async fn user_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) AS c FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("c"))
    }

    pub async fn create_user(&self, email: &str, password: &str) -> Result<User> {
        let email = users::normalize_email(email)?;
        let hash = users::hash_password(password)?;
        let res = sqlx::query(
            "INSERT INTO users (email, password_hash, notify_email) VALUES (?, ?, 1)",
        )
        .bind(&email)
        .bind(&hash)
        .execute(&self.pool)
        .await;
        match res {
            Ok(r) => Ok(User {
                id: r.last_insert_rowid(),
                email,
                notify_email: true,
            }),
            Err(sqlx::Error::Database(err)) if err.message().contains("UNIQUE") => {
                bail!("user already exists: {email}")
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let rows = sqlx::query(
            "SELECT id, email, notify_email FROM users ORDER BY email COLLATE NOCASE",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| User {
                id: row.get::<i64, _>("id"),
                email: row.get("email"),
                notify_email: row.get::<i64, _>("notify_email") != 0,
            })
            .collect())
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<(User, String)>> {
        let email = users::normalize_email(email)?;
        let row = sqlx::query(
            "SELECT id, email, password_hash, notify_email FROM users WHERE email = ? COLLATE NOCASE",
        )
        .bind(&email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| {
            (
                User {
                    id: row.get("id"),
                    email: row.get("email"),
                    notify_email: row.get::<i64, _>("notify_email") != 0,
                },
                row.get::<String, _>("password_hash"),
            )
        }))
    }

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, email, notify_email FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            notify_email: row.get::<i64, _>("notify_email") != 0,
        }))
    }

    pub async fn remove_user(&self, email: &str) -> Result<bool> {
        let email = users::normalize_email(email)?;
        let res = sqlx::query("DELETE FROM users WHERE email = ? COLLATE NOCASE")
            .bind(&email)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn set_password(&self, email: &str, password: &str) -> Result<bool> {
        let email = users::normalize_email(email)?;
        let hash = users::hash_password(password)?;
        let res = sqlx::query("UPDATE users SET password_hash = ? WHERE email = ? COLLATE NOCASE")
            .bind(&hash)
            .bind(&email)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn set_notify_email(&self, email: &str, enabled: bool) -> Result<bool> {
        let email = users::normalize_email(email)?;
        let res = sqlx::query(
            "UPDATE users SET notify_email = ? WHERE email = ? COLLATE NOCASE",
        )
        .bind(enabled as i64)
        .bind(&email)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn notification_emails(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT email FROM users WHERE notify_email = 1 ORDER BY email COLLATE NOCASE",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.get("email")).collect())
    }

    pub async fn add_fcm_token(&self, user_id: i64, token: &str) -> Result<()> {
        let token = token.trim();
        if token.is_empty() {
            bail!("empty FCM token");
        }
        sqlx::query(
            r#"
            INSERT INTO user_fcm_tokens (user_id, token)
            VALUES (?, ?)
            ON CONFLICT(token) DO UPDATE SET user_id = excluded.user_id
            "#,
        )
        .bind(user_id)
        .bind(token)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_fcm_tokens_for_user(&self, user_id: i64) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT token FROM user_fcm_tokens WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.get("token")).collect())
    }

    pub async fn list_all_fcm_tokens(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT token FROM user_fcm_tokens")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.get("token")).collect())
    }

    pub async fn remove_fcm_token(&self, token: &str) -> Result<bool> {
        let res = sqlx::query("DELETE FROM user_fcm_tokens WHERE token = ?")
            .bind(token.trim())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}

async fn migrate(pool: &SqlitePool) -> Result<()> {
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
    .execute(pool)
    .await
    .context("migrating power_events")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL COLLATE NOCASE UNIQUE,
            password_hash TEXT NOT NULL,
            notify_email INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await
    .context("migrating users")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_fcm_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await
    .context("migrating user_fcm_tokens")?;

    Ok(())
}
