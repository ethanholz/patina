-- Migration: Create devices table
-- This matches the Rust Device struct exactly
CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Optional text fields (Option<String> in Rust)
    name TEXT,
    friendly_id TEXT,
    current_screen_image TEXT,
    last_firmware_version TEXT,
    -- Required text fields (String in Rust)
    mac_address TEXT NOT NULL,
    api_key TEXT NOT NULL,
    image_format TEXT NOT NULL,
    -- Boolean field (bool in Rust)
    proxy_cloud BOOLEAN NOT NULL DEFAULT FALSE,
    -- Optional numeric fields (Option<f64>, Option<i32> in Rust)
    last_battery_voltage REAL,
    last_rssi_level INTEGER,
    -- Required integer fields (i32 in Rust)
    default_refresh_interval INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    rotate INTEGER NOT NULL,
    -- Datetime stored as text (String in Rust)
    created_at TEXT NOT NULL DEFAULT (datetime ('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime ('now'))
);

CREATE TABLE plugins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    data_payload TEXT,
    data_stale_minutes INTEGER,
    data_strategy TEXT,
    polling_url TEXT,
    polling_verb TEXT DEFAULT 'GET',
    polling_header TEXT,
    render_markup TEXT,
    render_markup_view TEXT,
    flux_icon_name TEXT,
    is_native BOOLEAN DEFAULT FALSE,
    data_payload_updated_at DATETIME,
    current_image TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    weekdays TEXT,
    active_from TIME,
    active_until TIME,
    refresh_time INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES devices (id) ON DELETE CASCADE
);

CREATE TABLE playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER NOT NULL,
    plugin_id INTEGER NOT NULL,
    order_index INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    last_displayed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (playlist_id) REFERENCES playlists (id) ON DELETE CASCADE,
    FOREIGN KEY (plugin_id) REFERENCES plugins (id) ON DELETE CASCADE
);
