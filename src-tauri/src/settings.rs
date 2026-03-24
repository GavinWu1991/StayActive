//! User settings: struct, defaults, validation, and file persistence.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User-configurable options for stay-active behavior. Matches data-model and contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Settings {
    pub interval_min_sec: u64,
    pub interval_max_sec: u64,
    #[serde(default = "default_random_interval")]
    pub random_interval: bool,
    #[serde(default = "default_move_pixels_min")]
    pub move_pixels_min: u32,
    #[serde(default = "default_move_pixels_max")]
    pub move_pixels_max: u32,
    #[serde(default = "default_simulate_move")]
    pub simulate_move: bool,
    #[serde(default = "default_simulate_click")]
    pub simulate_click: bool,
    #[serde(default = "default_click_button")]
    pub click_button: String,
    #[serde(default = "default_prevent_sleep")]
    pub prevent_sleep: bool,
    #[serde(default = "default_language")]
    pub language: String,
    /// Optional movement region; when enabled, overrides range-based movement.
    #[serde(default)]
    pub movement_region: MovementRegion,
}

/// Rectangular region (in physical pixels) where movement is constrained when enabled.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct MovementRegion {
    #[serde(default)]
    pub enabled: bool,
    pub x_min: Option<i32>,
    pub y_min: Option<i32>,
    pub x_max: Option<i32>,
    pub y_max: Option<i32>,
    pub display_ref: Option<String>,
}

fn default_move_pixels_min() -> u32 {
    1
}
fn default_move_pixels_max() -> u32 {
    3
}
fn default_random_interval() -> bool {
    true
}
fn default_simulate_move() -> bool {
    true
}
fn default_simulate_click() -> bool {
    true
}
fn default_click_button() -> String {
    "left".to_string()
}
fn default_prevent_sleep() -> bool {
    true
}
fn default_language() -> String {
    "en".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_min_sec: 45,
            interval_max_sec: 90,
            random_interval: true,
            move_pixels_min: 1,
            move_pixels_max: 3,
            simulate_move: true,
            simulate_click: true,
            click_button: "left".to_string(),
            prevent_sleep: true,
            language: "en".to_string(),
            movement_region: MovementRegion::default(),
        }
    }
}

const INTERVAL_CAP_SEC: u64 = 600;

/// Validate and clamp to allowed ranges per data-model.
pub fn validate(s: &mut Settings) {
    if s.interval_min_sec == 0 {
        s.interval_min_sec = 1;
    }
    if s.interval_min_sec > INTERVAL_CAP_SEC {
        s.interval_min_sec = INTERVAL_CAP_SEC;
    }
    if s.interval_max_sec < s.interval_min_sec {
        s.interval_max_sec = s.interval_min_sec;
    }
    if s.interval_max_sec > INTERVAL_CAP_SEC {
        s.interval_max_sec = INTERVAL_CAP_SEC;
    }
    if !s.random_interval {
        s.interval_max_sec = s.interval_min_sec;
    }
    if s.move_pixels_max < s.move_pixels_min {
        s.move_pixels_max = s.move_pixels_min;
    }

    if s.click_button != "left" && s.click_button != "right" {
        s.click_button = default_click_button();
    }

    validate_movement_region(&mut s.movement_region);
}

/// Basic validation for movement region configuration.
fn validate_movement_region(r: &mut MovementRegion) {
    if !r.enabled {
        return;
    }

    // Require all coordinates when enabled; otherwise disable region.
    let (Some(x_min), Some(y_min), Some(x_max), Some(y_max)) = (r.x_min, r.y_min, r.x_max, r.y_max)
    else {
        *r = MovementRegion::default();
        return;
    };

    // Enforce minimum size to avoid accidental tiny regions.
    let mut x_max = x_max;
    let mut y_max = y_max;

    if x_max <= x_min {
        x_max = x_min + 10;
    }
    if y_max <= y_min {
        y_max = y_min + 10;
    }
    if (x_max - x_min) < 10 {
        x_max = x_min + 10;
    }
    if (y_max - y_min) < 10 {
        y_max = y_min + 10;
    }

    r.x_min = Some(x_min);
    r.y_min = Some(y_min);
    r.x_max = Some(x_max);
    r.y_max = Some(y_max);
}

fn settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::data_local_dir().map(|d| d.join("StayActive").join("settings.json"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        dirs::config_dir().map(|d| d.join("StayActive").join("settings.json"))
    }
}

/// Load settings from disk; return defaults if missing or invalid.
pub fn load() -> Settings {
    let path = match settings_path() {
        Some(p) => p,
        None => return Settings::default(),
    };
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    let mut s: Settings = match serde_json::from_str(&contents) {
        Ok(s) => s,
        Err(_) => return Settings::default(),
    };
    validate(&mut s);
    s
}

/// Save settings to disk. Creates parent dir if needed.
pub fn save(s: &Settings) -> Result<(), String> {
    let path = settings_path().ok_or_else(|| "Could not resolve settings path".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let contents = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(&path, contents).map_err(|e| e.to_string())?;
    Ok(())
}
