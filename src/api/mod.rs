use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::models::device::Device;
use crate::models::state::AppState;

mod helpers;
use helpers::{extract_header_numeric, extract_header_string, extract_header_string_optional};

#[derive(Serialize)]
pub struct SetupResponse {
    pub api_key: String,
    pub friendly_id: String,
    pub image_url: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct DisplayResponse {
    image_url: String,
    image_url_timeout: u32,
    filename: String,
    refresh_rate: u32,
    reset_firmware: bool,
    update_firmware: bool,
    firmware_url: Option<String>,
    special_function: String,
}

#[derive(Deserialize, Debug)]
pub struct LogsRequest {
    log: Log,
}

#[derive(Deserialize, Debug)]
pub struct LogsResponse {
    logs_array: Vec<Log>,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub log_id: u32,
    pub creation_timestamp: i64,
    pub log_message: String,
    pub log_codeline: u32,
    pub device_status_stamp: DeviceStatusStamp,
    pub additional_info: AdditionalInfo,
    pub log_sourcefile: String,
}

#[derive(Deserialize, Debug)]
pub struct DeviceStatusStamp {
    pub wifi_status: String,
    pub wakeup_reason: String,
    pub current_fw_version: String,
    pub free_heap_size: u32,
    pub max_alloc_size: u32,
    pub special_function: String,
    pub refresh_rate: u32,
    pub battery_voltage: f64,
    pub time_since_last_sleep_start: u32,
    pub wifi_rssi_level: i32,
}

#[derive(Deserialize, Debug)]
pub struct AdditionalInfo {
    pub retry_attempt: u8,
}

#[derive(Serialize, Debug)]
pub struct CreateDeviceResponse {
    pub message: String,
}

pub async fn create_device_endpoint(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<CreateDeviceResponse>, StatusCode> {
    let mac_address = extract_header_string(&headers, "id")?;
    let api_key = extract_header_string(&headers, "access-token")?;
    let friendly_id = format!("device-{}", &mac_address[12..]);
    Device::create(
        &state.db,
        &mac_address,
        &api_key,
        &friendly_id,
        "TRMNL Device",
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = CreateDeviceResponse {
        message: "Succesfully added device".to_string(),
    };
    Ok(Json(response))
}

pub async fn log_endpoint(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LogsRequest>,
) -> StatusCode {
    let mac_address = extract_header_string(&headers, "id");
    if mac_address.is_err() {
        return StatusCode::BAD_REQUEST;
    }
    let mac_address = mac_address.unwrap();
    let api_key = extract_header_string(&headers, "access-token");
    if api_key.is_err() {
        return StatusCode::BAD_REQUEST;
    }
    let api_key = api_key.unwrap();
    let device = Device::find_by_credentials(&state.db, &mac_address, &api_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();
    if device.is_none() {
        return StatusCode::NOT_FOUND;
    }
    let time = chrono::DateTime::from_timestamp(payload.log.creation_timestamp, 0);
    if time.is_none() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let time = time.unwrap().naive_local();
    let log = payload.log;
    info!(
        "{} TIME: {} {} file:{}:{}",
        mac_address, time, log.log_message, log.log_sourcefile, log.log_codeline
    );
    return StatusCode::NO_CONTENT;
}

pub async fn display_endpoint(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<DisplayResponse>, StatusCode> {
    info!("display request received");
    let mac_address = extract_header_string(&headers, "id")?;
    let api_key = extract_header_string(&headers, "access-token")?;
    info!("mac_address {} api_key {}", mac_address, api_key);

    let device = Device::find_by_credentials(&state.db, &mac_address, &api_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    let device = device.ok_or(StatusCode::NOT_FOUND)?;
    info!("Device found!");
    if let (Some(rssi), Some(bat_volt), Some(fw_version)) = (
        extract_header_numeric::<i32>(&headers, "rssi"),
        extract_header_numeric::<f64>(&headers, "battery_voltage"),
        extract_header_string_optional(&headers, "fw-version"),
    ) {
        Device::update_device_info(&state.db, rssi, bat_volt, &fw_version)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        info!("device info updated!");
    }

    info!("attempting to find image");
    let (filename, image_path) = if let Some(image_uuid) = &device.current_screen_image {
        let use_bmp = device
            .last_firmware_version
            .as_ref()
            .map(|v| version_compare(v, "1.5.2") < 0)
            .unwrap_or(true);

        if use_bmp {
            (
                format!("{}.bmp", image_uuid),
                format!("images/generated/{}.bmp", image_uuid),
            )
        } else {
            (
                format!("{}.png", image_uuid),
                format!("images/generated/{}.png", image_uuid),
            )
        }
    } else {
        (
            "setup-logo.bmp".to_string(),
            "images/setup-logo.bmp".to_string(),
        )
    };
    info!("image found");
    let image_url = format!("{}/storage/{}", state.base_url, image_path);

    let resp = DisplayResponse {
        image_url: image_url.clone(),
        image_url_timeout: 15,
        filename,
        refresh_rate: device.default_refresh_interval as u32,
        reset_firmware: false,
        update_firmware: false,
        firmware_url: None,
        special_function: "sleep".to_string(),
    };
    info!("displaying {}", image_url);

    Ok(Json(resp))
}

pub async fn setup_endpoint(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<SetupResponse>, StatusCode> {
    info!("Received setup request!");
    let mac_address = extract_header_string(&headers, "id")?;

    info!("Attempting setup for {}", mac_address);
    let device = Device::find_by_mac(&state.db, &mac_address)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (api_key, friendly_id) = if let Some(device) = device {
        (
            device.api_key,
            device.friendly_id.unwrap_or_else(|| "unknown".to_string()),
        )
    } else {
        let api_key = uuid::Uuid::new_v4().to_string();
        let friendly_id = format!("device-{}", &mac_address[12..]);
        let _ = Device::create(
            &state.db,
            &mac_address,
            &api_key,
            &friendly_id,
            "TRMNL Device",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
        (api_key, friendly_id)
    };

    let resp = SetupResponse {
        api_key,
        friendly_id,
        image_url: format!("{}/storage/images/setup-logo.bmp", state.base_url),
        message: "Hello from TRMNL!".to_string(),
    };

    Ok(Json(resp))
}

// TODO: replace with semver crate?
fn version_compare(a: &str, b: &str) -> i8 {
    match a.cmp(b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

pub fn router() -> axum::Router<AppState> {
    Router::new()
        .route("/setup", get(setup_endpoint))
        .route("/display", get(display_endpoint))
        .route("/log", post(log_endpoint))
        .route("/add", post(create_device_endpoint))
}
