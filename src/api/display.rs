use crate::models::device::Device;
use serde::Serialize;

#[derive(Serialize)]
pub struct DisplayResponse {
    pub image_url: String,
    pub image_url_timeout: u32,
    pub filename: String,
    pub refresh_rate: u32,
    pub reset_firmware: bool,
    pub update_firmware: bool,
    pub firmware_url: Option<String>,
    pub special_function: String,
}

impl DisplayResponse {
    pub fn from_device(device: &Device, base_url: &str) -> Self {
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

        let image_url = format!("{}/storage/{}", base_url, image_path);

        DisplayResponse {
            image_url,
            image_url_timeout: 15,
            filename,
            refresh_rate: device.default_refresh_interval as u32,
            reset_firmware: false,
            update_firmware: false,
            firmware_url: None,
            special_function: "sleep".to_string(),
        }
    }
}

// TODO: replace with semver crate?
fn version_compare(a: &str, b: &str) -> i8 {
    match a.cmp(b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn create_test_device() -> Device {
        Device {
            id: 1,
            name: Some("Test Device".to_string()),
            mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
            api_key: "test_api_key".to_string(),
            friendly_id: Some("test_device".to_string()),
            proxy_cloud: false,
            current_screen_image: None,
            last_battery_voltage: Some(3.7),
            last_rssi_level: Some(-50),
            last_firmware_version: None,
            default_refresh_interval: 60,
            width: 800,
            height: 480,
            rotate: 0,
            image_format: "png".to_string(),
            created_at: NaiveDateTime::parse_from_str("2001-09-09 01:46:40", "%Y-%m-%d %H:%M:%S").unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2001-09-09 01:46:40", "%Y-%m-%d %H:%M:%S").unwrap(),
        }
    }

    #[test]
    fn test_display_response_from_device_with_no_image() {
        let device = create_test_device();
        let base_url = "https://example.com";

        let response = DisplayResponse::from_device(&device, base_url);

        assert_eq!(response.image_url, "https://example.com/storage/images/setup-logo.bmp");
        assert_eq!(response.image_url_timeout, 15);
        assert_eq!(response.filename, "setup-logo.bmp");
        assert_eq!(response.refresh_rate, 60);
        assert_eq!(response.reset_firmware, false);
        assert_eq!(response.update_firmware, false);
        assert_eq!(response.firmware_url, None);
        assert_eq!(response.special_function, "sleep");
    }

    #[test]
    fn test_display_response_from_device_with_image_old_firmware() {
        let mut device = create_test_device();
        device.current_screen_image = Some("test-uuid-123".to_string());
        device.last_firmware_version = Some("1.4.0".to_string());
        let base_url = "https://example.com";

        let response = DisplayResponse::from_device(&device, base_url);

        assert_eq!(response.image_url, "https://example.com/storage/images/generated/test-uuid-123.bmp");
        assert_eq!(response.filename, "test-uuid-123.bmp");
        assert_eq!(response.refresh_rate, 60);
    }

    #[test]
    fn test_display_response_from_device_with_image_new_firmware() {
        let mut device = create_test_device();
        device.current_screen_image = Some("test-uuid-456".to_string());
        device.last_firmware_version = Some("1.6.0".to_string());
        let base_url = "https://example.com";

        let response = DisplayResponse::from_device(&device, base_url);

        assert_eq!(response.image_url, "https://example.com/storage/images/generated/test-uuid-456.png");
        assert_eq!(response.filename, "test-uuid-456.png");
        assert_eq!(response.refresh_rate, 60);
    }

    #[test]
    fn test_display_response_from_device_with_image_exact_firmware_version() {
        let mut device = create_test_device();
        device.current_screen_image = Some("test-uuid-789".to_string());
        device.last_firmware_version = Some("1.5.2".to_string());
        let base_url = "https://example.com";

        let response = DisplayResponse::from_device(&device, base_url);

        // Version 1.5.2 should use PNG (not less than 1.5.2)
        assert_eq!(response.image_url, "https://example.com/storage/images/generated/test-uuid-789.png");
        assert_eq!(response.filename, "test-uuid-789.png");
    }

    #[test]
    fn test_display_response_from_device_with_image_no_firmware_version() {
        let mut device = create_test_device();
        device.current_screen_image = Some("test-uuid-no-fw".to_string());
        device.last_firmware_version = None;
        let base_url = "https://example.com";

        let response = DisplayResponse::from_device(&device, base_url);

        // No firmware version should default to BMP
        assert_eq!(response.image_url, "https://example.com/storage/images/generated/test-uuid-no-fw.bmp");
        assert_eq!(response.filename, "test-uuid-no-fw.bmp");
    }

    #[test]
    fn test_display_response_custom_refresh_rate() {
        let mut device = create_test_device();
        device.default_refresh_interval = 120;
        let base_url = "https://example.com";

        let response = DisplayResponse::from_device(&device, base_url);

        assert_eq!(response.refresh_rate, 120);
    }

    #[test]
    fn test_display_response_different_base_url() {
        let device = create_test_device();
        let base_url = "http://localhost:3000";

        let response = DisplayResponse::from_device(&device, base_url);

        assert_eq!(response.image_url, "http://localhost:3000/storage/images/setup-logo.bmp");
    }

    #[test]
    fn test_version_compare_equal() {
        assert_eq!(version_compare("1.5.2", "1.5.2"), 0);
        assert_eq!(version_compare("", ""), 0);
        assert_eq!(version_compare("abc", "abc"), 0);
    }

    #[test]
    fn test_version_compare_less_than() {
        assert_eq!(version_compare("1.4.0", "1.5.2"), -1);
        assert_eq!(version_compare("1.5.1", "1.5.2"), -1);
        assert_eq!(version_compare("0.9.9", "1.0.0"), -1);
        assert_eq!(version_compare("", "a"), -1);
    }

    #[test]
    fn test_version_compare_greater_than() {
        assert_eq!(version_compare("1.6.0", "1.5.2"), 1);
        assert_eq!(version_compare("2.0.0", "1.9.9"), 1);
        assert_eq!(version_compare("1.5.3", "1.5.2"), 1);
        assert_eq!(version_compare("a", ""), 1);
    }

    #[test]
    fn test_version_compare_string_comparison() {
        // Note: This is string comparison, not semantic version comparison
        // "10" < "2" in string comparison
        assert_eq!(version_compare("1.10.0", "1.2.0"), -1);
        assert_eq!(version_compare("1.2.0", "1.10.0"), 1);
    }

    #[test]
    fn test_display_response_serialization() {
        let device = create_test_device();
        let response = DisplayResponse::from_device(&device, "https://test.com");
        
        // Test that it can be serialized (this will panic if there are issues)
        let json = serde_json::to_string(&response).expect("Should serialize successfully");
        assert!(json.contains("setup-logo.bmp"));
        assert!(json.contains("https://test.com"));
        assert!(json.contains("\"refresh_rate\":60"));
        assert!(json.contains("\"image_url_timeout\":15"));
        assert!(json.contains("\"reset_firmware\":false"));
        assert!(json.contains("\"update_firmware\":false"));
        assert!(json.contains("\"firmware_url\":null"));
        assert!(json.contains("\"special_function\":\"sleep\""));
    }
}
