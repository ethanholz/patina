use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use sqlx::{prelude::*, sqlite::SqliteQueryResult};

#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: i64,
    pub name: Option<String>,
    pub mac_address: String,
    pub api_key: String,
    pub friendly_id: Option<String>,
    pub proxy_cloud: bool,
    pub current_screen_image: Option<String>,
    pub last_battery_voltage: Option<f64>,
    pub last_rssi_level: Option<i32>,
    pub last_firmware_version: Option<String>,
    pub default_refresh_interval: i32,
    pub width: i32,
    pub height: i32,
    pub rotate: i32,
    pub image_format: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Device {
    pub async fn find_by_credentials(
        pool: &sqlx::SqlitePool,
        mac_address: &str,
        api_key: &str,
    ) -> Result<Option<Device>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM devices WHERE mac_address = ? AND api_key = ?")
            .bind(mac_address)
            .bind(api_key)
            .fetch_optional(pool)
            .await
    }

    pub async fn update_device_info(
        pool: &sqlx::SqlitePool,
        rssi: i32,
        bat_volt: f64,
        fw_version: &str,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE devices SET last_rssi_level = ?, last_battery_voltage = ?, last_firmware_version = ?, updated_at CURRENT_TIMESTAMP WHERE id = ?")
            .bind(rssi)
            .bind(bat_volt)
            .bind(fw_version)
            .execute(pool)
            .await
    }

    pub async fn find_by_mac(
        pool: &sqlx::SqlitePool,
        mac_address: &str,
    ) -> Result<Option<Device>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM devices WHERE mac_address = ?")
            .bind(mac_address)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(
        pool: &sqlx::SqlitePool,
        mac_address: &str,
        api_key: &str,
        friendly_id: &str,
        name: &str,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query(
            "INSERT INTO devices (mac_address, api_key, friendly_id, name, image_format, default_refresh_interval, width, height, rotate, proxy_cloud, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
        )
        .bind(mac_address)
        .bind(api_key)
        .bind(friendly_id)
        .bind(name)
        // image format
        .bind("png")
        // refresh interval
        .bind(60)
        // width
        .bind(800)
        // height
        .bind(480)
        // rotate
        .bind(0)
        // proxy cloud
        .bind(false)
        .execute(pool)
        .await
    }
}
