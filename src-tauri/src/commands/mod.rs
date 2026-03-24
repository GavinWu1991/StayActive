//! Tauri commands: start/stop stay-active, get state, permission, settings, timer.

#[cfg(debug_assertions)]
macro_rules! dev_log {
    ($($t:tt)*) => {{
        let _msg = format!("[StayActive:cmd] {}", format!($($t)*));
        eprintln!("{}", _msg);
        crate::dev_log_write(&_msg);
    }}
}
#[cfg(not(debug_assertions))]
macro_rules! dev_log {
    ($($t:tt)*) => {}
}

use crate::automation;
use crate::permission;
use crate::settings::{self, Settings, MovementRegion};
use crate::timer;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::Mutex;

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TimerState {
    pub active: bool,
    pub mode: Option<String>,
    pub duration_secs: Option<u64>,
    pub remaining_secs: Option<u64>,
}

pub struct LoopHandle(pub Arc<Mutex<Option<std::thread::JoinHandle<()>>>>);

#[tauri::command]
pub fn check_accessibility_permission() -> Result<bool, String> {
    Ok(permission::check_accessibility_permission())
}

#[tauri::command]
pub fn open_system_preferences_accessibility() -> Result<(), String> {
    permission::open_system_preferences_accessibility()
}

#[tauri::command]
pub fn get_stay_active_state(_handle: State<'_, LoopHandle>) -> Result<bool, String> {
    Ok(automation::is_running())
}

#[tauri::command]
pub async fn start_stay_active(app: AppHandle, handle: State<'_, LoopHandle>) -> Result<(), String> {
    dev_log!("start_stay_active enter, is_running={}", automation::is_running());
    if !permission::check_accessibility_permission() {
        let _ = app.emit("permission_required", ());
        // Trigger system prompt and open Accessibility pane so user can add the app.
        permission::request_accessibility_prompt();
        let _ = permission::open_system_preferences_accessibility();
        let mut msg = "Accessibility permission not granted. A system dialog or System Settings should open—add this app to the list and enable it, then try Start again. If no dialog appears, you may be running the dev binary (e.g. tauri dev): use \"npm run dev:app\" and add the .app to Accessibility, then run dev:app again.".to_string();
        #[cfg(all(debug_assertions, target_os = "macos"))]
        {
            msg.push_str("\n\nPath to the .app: <project>/src-tauri/target/debug/bundle/macos/StayActive.app. Use Cmd+Shift+G in the Add dialog to open that folder.");
        }
        return Err(msg);
    }
    if automation::is_running() {
        dev_log!("start_stay_active skip (already running)");
        return Ok(());
    }
    automation::set_running(true);
    automation::clear_cancelled();
    dev_log!("start_stay_active set_running(true), spawning loop");
    let s = settings::load();
    let join = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            automation::run_loop_blocking(
                s.interval_min_sec,
                s.interval_max_sec,
                s.random_interval,
                s.move_pixels_min,
                s.move_pixels_max,
                s.simulate_move,
                s.simulate_click,
                s.click_button,
                s.prevent_sleep,
            );
        }));
        if let Err(e) = result {
            dev_log!("automation thread panic: {:?}", e);
            automation::set_running(false);
        }
    });
    let mut h = handle.0.lock().await;
    *h = Some(join);
    let _ = app.emit("stay_active_state_changed", serde_json::json!({ "active": true }));
    dev_log!("start_stay_active exit, is_running={}", automation::is_running());
    Ok(())
}

#[tauri::command]
pub async fn stop_stay_active(app: AppHandle, handle: State<'_, LoopHandle>) -> Result<(), String> {
    dev_log!("stop_stay_active enter, is_running={}", automation::is_running());
    automation::cancel();
    timer::clear();
    automation::set_running(false);
    let _ = app.emit("stay_active_state_changed", serde_json::json!({ "active": false }));
    // Join automation thread in background so this command can return immediately.
    let mut h = handle.0.lock().await;
    if let Some(join) = h.take() {
        std::thread::spawn(move || {
            let _ = join.join();
        });
    }
    dev_log!("stop_stay_active exit, is_running={}", automation::is_running());
    Ok(())
}

#[tauri::command]
pub fn get_settings() -> Result<Settings, String> {
    Ok(settings::load())
}

#[tauri::command]
pub async fn set_settings(
    app: AppHandle,
    handle: State<'_, LoopHandle>,
    s: Settings,
) -> Result<(), String> {
    let mut s = s;
    settings::validate(&mut s);
    let old_lang = settings::load().language.clone();
    settings::save(&s)?;
    // If language changed, refresh menu
    #[cfg(any(windows, target_os = "macos"))]
    if s.language != old_lang {
        crate::refresh_tray_menu(&app);
    }
    // If stay-active is on, restart loop so new settings apply immediately.
    if automation::is_running() {
        let _ = stop_stay_active(app.clone(), handle).await;
        let app2 = app.clone();
        let handle2 = app2.state::<LoopHandle>();
        let _ = start_stay_active(app, handle2).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_language(app: AppHandle, lang: String) -> Result<(), String> {
    if lang != "en" && lang != "zh" {
        return Err("Language must be 'en' or 'zh'".to_string());
    }
    let mut s = settings::load();
    s.language = lang;
    settings::save(&s)?;
    #[cfg(any(windows, target_os = "macos"))]
    crate::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub async fn start_region_selection(app: AppHandle) -> Result<(), String> {
    // Show region picker overlay window.
    if let Some(win) = app.get_webview_window("region-picker") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "region-picker", WebviewUrl::App("index.html".into()))
        .title("Select movement region")
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_movement_region(
    app: AppHandle,
    handle: State<'_, LoopHandle>,
    enabled: bool,
    display_ref: Option<String>,
    x_min: Option<f64>,
    y_min: Option<f64>,
    x_max: Option<f64>,
    y_max: Option<f64>,
) -> Result<(), String> {
    let mut s = settings::load();
    s.movement_region.enabled = enabled;
    s.movement_region.display_ref = display_ref;
    s.movement_region.x_min = x_min.map(|v| v.round() as i32);
    s.movement_region.y_min = y_min.map(|v| v.round() as i32);
    s.movement_region.x_max = x_max.map(|v| v.round() as i32);
    s.movement_region.y_max = y_max.map(|v| v.round() as i32);
    settings::validate(&mut s);
    settings::save(&s)?;

    // Notify UI so it can refresh preview state.
    let _ = app.emit(
        "movement_region_selected",
        serde_json::json!({
            "enabled": s.movement_region.enabled,
            "display_ref": s.movement_region.display_ref,
            "x_min": s.movement_region.x_min,
            "y_min": s.movement_region.y_min,
            "x_max": s.movement_region.x_max,
            "y_max": s.movement_region.y_max,
        }),
    );

    // If stay-active is on, restart loop so new region applies immediately.
    if automation::is_running() {
        let _ = stop_stay_active(app.clone(), handle).await;
        let app2 = app.clone();
        let handle2 = app2.state::<LoopHandle>();
        let _ = start_stay_active(app, handle2).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn clear_movement_region(app: AppHandle, handle: State<'_, LoopHandle>) -> Result<(), String> {
    let mut s = settings::load();
    s.movement_region = MovementRegion::default();
    settings::save(&s)?;

    let _ = app.emit("movement_region_selected", serde_json::json!({ "enabled": false }));

    if automation::is_running() {
        let _ = stop_stay_active(app.clone(), handle).await;
        let app2 = app.clone();
        let handle2 = app2.state::<LoopHandle>();
        let _ = start_stay_active(app, handle2).await;
    }

    Ok(())
}

#[tauri::command]
pub fn get_timer_state() -> Result<TimerState, String> {
    let (active, remaining_secs, mode_u, duration_secs) = timer::state();
    let mode = if mode_u == 1 {
        Some("preset".to_string())
    } else if mode_u == 2 {
        Some("custom".to_string())
    } else {
        None
    };
    Ok(TimerState {
        active,
        mode,
        duration_secs: if duration_secs > 0 { Some(duration_secs) } else { None },
        remaining_secs: if active && remaining_secs > 0 {
            Some(remaining_secs)
        } else {
            None
        },
    })
}

#[tauri::command]
pub async fn set_timer_preset(app: AppHandle, _handle: State<'_, LoopHandle>, duration_secs: u64) -> Result<(), String> {
    dev_log!("set_timer_preset duration_secs={}", duration_secs);
    const ALLOWED: [u64; 5] = [600, 1800, 3600, 7200, 10800];
    if !ALLOWED.contains(&duration_secs) {
        return Err(format!("Invalid preset: use 600, 1800, 3600, 7200, or 10800 (got {})", duration_secs));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let end_sec = now + duration_secs;
    timer::set_end(end_sec, 1, duration_secs);
    let app_guard = app.clone();
    tauri::async_runtime::spawn(async move {
        // Refresh right away so menu shows countdown immediately.
        #[cfg(any(windows, target_os = "macos"))]
        crate::refresh_tray_menu(&app_guard);

        loop {
            if timer::is_cancelled() {
                dev_log!("timer preset task cancelled");
                return;
            }
            if timer::is_expired() {
                dev_log!("timer preset expired");
                timer::clear();
                let h = app_guard.state::<LoopHandle>();
                let _ = stop_stay_active(app_guard.clone(), h).await;
                #[cfg(any(windows, target_os = "macos"))]
                crate::update_tray_icon(&app_guard);
                #[cfg(any(windows, target_os = "macos"))]
                crate::refresh_tray_menu(&app_guard);
                let _ = app_guard.emit("timer_ended", serde_json::json!({}));
                return;
            }

            let (active, remaining_secs, _mode, _dur) = timer::state();
            if !active {
                return;
            }

            // Dynamic refresh schedule:
            // - >60s: refresh every 60s
            // - 10s < remaining <= 60s: refresh every 1s
            // - remaining <= 10s: refresh every 500ms
            let sleep_dur = if remaining_secs > 60 {
                std::time::Duration::from_secs(60)
            } else if remaining_secs > 10 {
                std::time::Duration::from_secs(1)
            } else {
                std::time::Duration::from_millis(500)
            };
            tokio::time::sleep(sleep_dur).await;

            #[cfg(any(windows, target_os = "macos"))]
            crate::refresh_tray_menu(&app_guard);
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_timer() -> Result<(), String> {
    dev_log!("cancel_timer");
    timer::set_cancelled(true);
    timer::clear();
    Ok(())
}

/// Compute unix timestamp (seconds) for "today at hour:minute" local, or tomorrow if that time has passed.
fn end_sec_from_hour_minute(hour: u32, minute: u32) -> Result<u64, String> {
    let now = chrono::Local::now();
    let date = now.date_naive();
    let target_naive = date
        .and_hms_opt(hour, minute, 0)
        .ok_or_else(|| "Invalid hour/minute".to_string())?;
    let target_local = target_naive
        .and_local_timezone(chrono::Local)
        .single()
        .ok_or_else(|| "Invalid local time".to_string())?;
    let end_dt = if target_local <= now {
        target_local + chrono::Duration::days(1)
    } else {
        target_local
    };
    Ok(end_dt.timestamp().max(0) as u64)
}

#[tauri::command]
pub async fn set_timer_custom(app: AppHandle, _handle: State<'_, LoopHandle>, hour: u32, minute: u32) -> Result<(), String> {
    dev_log!("set_timer_custom hour={} minute={}", hour, minute);
    if hour > 23 || minute > 59 {
        return Err("Hour must be 0–23, minute 0–59".to_string());
    }
    let end_sec = end_sec_from_hour_minute(hour, minute)?;
    timer::set_end(end_sec, 2, 0); // mode 2 = custom, duration not used
    let app_guard = app.clone();
    tauri::async_runtime::spawn(async move {
        // Refresh right away so menu shows countdown immediately.
        #[cfg(any(windows, target_os = "macos"))]
        crate::refresh_tray_menu(&app_guard);

        loop {
            if timer::is_cancelled() {
                dev_log!("timer custom task cancelled");
                return;
            }
            if timer::is_expired() {
                dev_log!("timer custom expired");
                timer::clear();
                let h = app_guard.state::<LoopHandle>();
                let _ = stop_stay_active(app_guard.clone(), h).await;
                #[cfg(any(windows, target_os = "macos"))]
                crate::update_tray_icon(&app_guard);
                #[cfg(any(windows, target_os = "macos"))]
                crate::refresh_tray_menu(&app_guard);
                let _ = app_guard.emit("timer_ended", serde_json::json!({}));
                return;
            }

            let (active, remaining_secs, _mode, _dur) = timer::state();
            if !active {
                return;
            }

            // Dynamic refresh schedule:
            // - >60s: refresh every 60s
            // - 10s < remaining <= 60s: refresh every 1s
            // - remaining <= 10s: refresh every 500ms
            let sleep_dur = if remaining_secs > 60 {
                std::time::Duration::from_secs(60)
            } else if remaining_secs > 10 {
                std::time::Duration::from_secs(1)
            } else {
                std::time::Duration::from_millis(500)
            };
            tokio::time::sleep(sleep_dur).await;

            #[cfg(any(windows, target_os = "macos"))]
            crate::refresh_tray_menu(&app_guard);
        }
    });
    Ok(())
}
