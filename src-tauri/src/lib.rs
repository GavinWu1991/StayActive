#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod automation;
mod commands;
mod permission;
mod settings;
mod timer;

use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIcon, TrayIconBuilder},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

#[cfg(any(windows, target_os = "macos"))]
const TRAY_ICON_ON: Image<'_> = tauri::include_image!("icons/tray-on.png");
#[cfg(any(windows, target_os = "macos"))]
const TRAY_ICON_OFF: Image<'_> = tauri::include_image!("icons/tray-off.png");

/// Target tray icon size (points). Use 44 for @2x retina so content fills the menu bar slot.
#[cfg(any(windows, target_os = "macos"))]
const TRAY_ICON_SIZE: u32 = 44;

/// Scales icon content to fill more of the tray slot: crop transparent edges, then resize content to fill (TRAY_ICON_SIZE - padding).
#[cfg(any(windows, target_os = "macos"))]
fn scale_icon_for_tray(icon: &Image<'_>) -> Image<'static> {
    use image::{imageops, RgbaImage};
    let w = icon.width();
    let h = icon.height();
    let rgba = icon.rgba();
    let img = match RgbaImage::from_raw(w, h, rgba.to_vec()) {
        Some(i) => i,
        None => return Image::new_owned(rgba.to_vec(), w, h),
    };
    const ALPHA_THRESHOLD: u8 = 12;
    let (mut x_min, mut y_min, mut x_max, mut y_max) = (w, h, 0u32, 0u32);
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p[3] > ALPHA_THRESHOLD {
                x_min = x_min.min(x);
                y_min = y_min.min(y);
                x_max = x_max.max(x);
                y_max = y_max.max(y);
            }
        }
    }
    if x_max < x_min || y_max < y_min {
        return Image::new_owned(rgba.to_vec(), w, h);
    }
    let cw = x_max - x_min + 1;
    let ch = y_max - y_min + 1;
    let cropped = imageops::crop_imm(&img, x_min, y_min, cw, ch).to_image();
    const PAD: u32 = 2;
    let size = TRAY_ICON_SIZE.saturating_sub(PAD * 2);
    let scaled = imageops::resize(
        &cropped,
        size,
        size,
        imageops::FilterType::Lanczos3,
    );
    let mut out = RgbaImage::new(TRAY_ICON_SIZE, TRAY_ICON_SIZE);
    let dx = (TRAY_ICON_SIZE - scaled.width()) / 2;
    let dy = (TRAY_ICON_SIZE - scaled.height()) / 2;
    imageops::overlay(&mut out, &scaled, dx.into(), dy.into());
    Image::new_owned(out.into_raw(), TRAY_ICON_SIZE, TRAY_ICON_SIZE)
}

/// Menu translations based on language setting.
#[cfg(any(windows, target_os = "macos"))]
struct MenuTranslations {
    stay_active: &'static str,
    timer: &'static str,
    timer_10min: &'static str,
    timer_30min: &'static str,
    timer_1h: &'static str,
    timer_2h: &'static str,
    timer_3h: &'static str,
    timer_custom: &'static str,
    settings: &'static str,
    quit: &'static str,
    remaining: &'static str,
    cancel_countdown_title: &'static str,
    cancel_countdown_message: &'static str,
}

#[cfg(any(windows, target_os = "macos"))]
fn get_menu_translations(lang: &str) -> MenuTranslations {
    match lang {
        "zh" => MenuTranslations {
            stay_active: "保持活跃",
            timer: "定时器",
            timer_10min: "10 分钟",
            timer_30min: "30 分钟",
            timer_1h: "1 小时",
            timer_2h: "2 小时",
            timer_3h: "3 小时",
            timer_custom: "自定义…",
            settings: "设置…",
            quit: "退出",
            remaining: "剩余",
            cancel_countdown_title: "取消倒计时？",
            cancel_countdown_message: "停止自动停止定时器，保持 Stay Active 运行直到手动关闭。",
        },
        _ => MenuTranslations {
            stay_active: "Stay Active",
            timer: "Timer",
            timer_10min: "10 minutes",
            timer_30min: "30 minutes",
            timer_1h: "1 hour",
            timer_2h: "2 hours",
            timer_3h: "3 hours",
            timer_custom: "Custom…",
            settings: "Settings...",
            quit: "Quit",
            remaining: "remaining",
            cancel_countdown_title: "Cancel countdown?",
            cancel_countdown_message: "Stop the auto-stop timer and keep Stay Active running until you turn it off.",
        },
    }
}

/// Formats remaining seconds as "M:SS remaining" for menu label.
#[cfg(any(windows, target_os = "macos"))]
fn format_remaining(secs: u64, lang: &str) -> String {
    let m = secs / 60;
    let s = secs % 60;
    let t = get_menu_translations(lang);
    format!("{}:{:02} {}", m, s, t.remaining)
}

/// Writes a line to the debug log file (debug build only). Path: ~/Library/Logs/StayActive/debug.log
#[cfg(debug_assertions)]
pub fn dev_log_write(line: &str) {
    use std::io::Write;
    let _ = (|| {
        let home = std::env::var("HOME").ok().unwrap_or_else(|| "/tmp".into());
        let dir = std::path::Path::new(&home).join("Library/Logs/StayActive");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("debug.log");
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok()?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        writeln!(f, "[{}] {}", ts, line).ok()?;
        Some(())
    })();
}
#[cfg(not(debug_assertions))]
pub fn dev_log_write(_line: &str) {}

/// Dev-only debug log: stderr + file (no-op in release).
#[cfg(debug_assertions)]
macro_rules! dev_log {
    ($($t:tt)*) => {{
        let _msg = format!("[StayActive] {}", format!($($t)*));
        eprintln!("{}", _msg);
        crate::dev_log_write(&_msg);
    }}
}
#[cfg(not(debug_assertions))]
macro_rules! dev_log {
    ($($t:tt)*) => {}
}

/// Builds the full tray menu:
///   Stay Active | ---- | [Remaining if enabled] | Timer (submenu) | ---- | Settings | Quit
#[cfg(any(windows, target_os = "macos"))]
fn build_tray_menu(app: &tauri::AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    use tauri::menu::IsMenuItem;
    use crate::settings;
    let running = automation::is_running();
    let (active, remaining_secs, _mode, _dur) = timer::state();
    let s = settings::load();
    let lang = s.language.as_str();
    let t = get_menu_translations(lang);
    dev_log!("build_tray_menu: is_running={} timer_active={} remaining_secs={} lang={}", running, active, remaining_secs, lang);
    let toggle_i = CheckMenuItem::with_id(
        app,
        "toggle",
        t.stay_active,
        true,
        running,
        None::<&str>,
    )?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let t10 = MenuItem::with_id(app, "timer_600", t.timer_10min, true, None::<&str>)?;
    let t30 = MenuItem::with_id(app, "timer_1800", t.timer_30min, true, None::<&str>)?;
    let t1h = MenuItem::with_id(app, "timer_3600", t.timer_1h, true, None::<&str>)?;
    let t2h = MenuItem::with_id(app, "timer_7200", t.timer_2h, true, None::<&str>)?;
    let t3h = MenuItem::with_id(app, "timer_10800", t.timer_3h, true, None::<&str>)?;
    let t_custom = MenuItem::with_id(app, "timer_custom", t.timer_custom, true, None::<&str>)?;
    let timer_submenu = Submenu::with_items(app, t.timer, true, &[&t10, &t30, &t1h, &t2h, &t3h, &t_custom])?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let settings_i = MenuItem::with_id(app, "settings", t.settings, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", t.quit, true, None::<&str>)?;
    let menu = if active && remaining_secs > 0 {
        let remaining_i = MenuItem::with_id(app, "timer_cancel", format_remaining(remaining_secs, lang), true, None::<&str>)?;
        Menu::with_items(
            app,
            &[
                &toggle_i as &dyn IsMenuItem<_>,
                &sep1,
                &remaining_i,
                &timer_submenu,
                &sep2,
                &settings_i,
                &quit_i,
            ],
        )?
    } else {
        Menu::with_items(
            app,
            &[
                &toggle_i as &dyn IsMenuItem<_>,
                &sep1,
                &timer_submenu,
                &sep2,
                &settings_i,
                &quit_i,
            ],
        )?
    };
    Ok(menu)
}

/// Rebuilds and sets the tray menu (e.g. after timer tick or cancel). Call from timer task or menu handler.
#[cfg(any(windows, target_os = "macos"))]
pub(crate) fn refresh_tray_menu(app: &tauri::AppHandle) {
    dev_log!("refresh_tray_menu called");
    if let Some(tray) = app.try_state::<TrayIcon<tauri::Wry>>() {
        if let Ok(menu) = build_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

/// Updates the tray icon to match stay-active state. Call after start/stop or timer_ended.
#[cfg(any(windows, target_os = "macos"))]
pub(crate) fn update_tray_icon(app: &tauri::AppHandle) {
    let running = automation::is_running();
    dev_log!("update_tray_icon: is_running={}", running);
    if let Some(tray) = app.try_state::<TrayIcon<tauri::Wry>>() {
        let icon = if running {
            scale_icon_for_tray(&TRAY_ICON_ON)
        } else {
            scale_icon_for_tray(&TRAY_ICON_OFF)
        };
        let _ = tray.set_icon(Some(icon));
        #[cfg(target_os = "macos")]
        let _ = tray.set_icon_as_template(true);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::LoopHandle(Arc::new(Mutex::new(None::<std::thread::JoinHandle<()>>))))
        .invoke_handler(tauri::generate_handler![
            commands::check_accessibility_permission,
            commands::open_system_preferences_accessibility,
            commands::get_stay_active_state,
            commands::get_settings,
            commands::set_settings,
            commands::set_language,
            commands::start_region_selection,
            commands::set_movement_region,
            commands::clear_movement_region,
            commands::get_timer_state,
            commands::set_timer_preset,
            commands::set_timer_custom,
            commands::cancel_timer,
            commands::start_stay_active,
            commands::stop_stay_active,
        ])
        .setup(|app| {
            #[cfg(any(windows, target_os = "macos"))]
            {
                let menu = build_tray_menu(app.handle())?;
                let initial_icon = if automation::is_running() {
                    scale_icon_for_tray(&TRAY_ICON_ON)
                } else {
                    scale_icon_for_tray(&TRAY_ICON_OFF)
                };
                let tray = TrayIconBuilder::new()
                    .icon(initial_icon)
                    .menu(&menu)
                    .on_menu_event(move |app, event| {
                        dev_log!("menu_event: id={}", event.id.as_ref());
                        match event.id.as_ref() {
                            "toggle" => {
                                tauri::async_runtime::block_on(async {
                                    let handle = app.state::<commands::LoopHandle>();
                                    let running = automation::is_running();
                                    if running {
                                        let _ = commands::stop_stay_active(app.clone(), handle).await;
                                    } else {
                                        let _ = app.emit("stay_active_state_changed", serde_json::json!({ "active": true }));
                                        if let Err(e) = commands::start_stay_active(app.clone(), handle).await {
                                            eprintln!("start_stay_active: {}", e);
                                        }
                                    }
                                    update_tray_icon(app);
                                    refresh_tray_menu(app);
                                });
                            }
                            "timer_custom" => {
                                if let Some(win) = app.get_webview_window("timer-picker") {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                } else {
                                    let _ = WebviewWindowBuilder::new(app, "timer-picker", WebviewUrl::App("index.html".into()))
                                        .title("Set auto-stop time")
                                        .inner_size(320.0, 160.0)
                                        .build();
                                }
                            }
                            "timer_600" | "timer_1800" | "timer_3600" | "timer_7200" | "timer_10800" => {
                                let secs: u64 = match event.id.as_ref() {
                                    "timer_600" => 600,
                                    "timer_1800" => 1800,
                                    "timer_3600" => 3600,
                                    "timer_7200" => 7200,
                                    "timer_10800" => 10800,
                                    _ => return,
                                };
                                tauri::async_runtime::block_on(async {
                                    let handle = app.state::<commands::LoopHandle>();
                                    let _ = commands::set_timer_preset(app.clone(), handle, secs).await;
                                    refresh_tray_menu(app);
                                });
                            }
                            "timer_cancel" => {
                                let app_clone = app.clone();
                                let s = settings::load();
                                let t = get_menu_translations(s.language.as_str());
                                app.dialog()
                                    .message(t.cancel_countdown_message)
                                    .title(t.cancel_countdown_title)
                                    .buttons(MessageDialogButtons::OkCancel)
                                    .show(move |confirmed| {
                                        if confirmed {
                                            let _ = commands::cancel_timer();
                                            refresh_tray_menu(&app_clone);
                                        }
                                    });
                            }
                            "settings" => {
                                if let Some(win) = app.get_webview_window("settings") {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                } else {
                                    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
                                        .title("StayActive Settings")
                                        .inner_size(520.0, 650.0)
                                        .build();
                                }
                            }
                            "quit" => {
                                tauri::async_runtime::block_on(async {
                                    let handle = app.state::<commands::LoopHandle>();
                                    let _ = commands::stop_stay_active(app.clone(), handle).await;
                                });
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .build(app)?;
                #[cfg(target_os = "macos")]
                let _ = tray.set_icon_as_template(true);
                app.manage(tray);
            }

            if !permission::check_accessibility_permission() {
                let _ = app.emit("permission_required", ());
            } else {
                let _ = app.emit("permission_granted", ());
            }

            // Permission-revoked detection: require two consecutive failures (with 2s gap) before
            // stopping, to avoid transient AXIsProcessTrustedWithOptions false causing unexpected stop.
            let app_guard = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    if !automation::is_running() {
                        update_tray_icon(&app_guard);
                        continue;
                    }
                    if permission::check_accessibility_permission() {
                        continue;
                    }
                    // Second check after 2s to avoid transient false
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    if permission::check_accessibility_permission() {
                        continue;
                    }
                    automation::cancel();
                    let handle = app_guard.state::<commands::LoopHandle>();
                    let join_handle = {
                        let mut h = handle.0.lock().await;
                        h.take()
                    };
                    if let Some(join) = join_handle {
                        let _ = tokio::task::spawn_blocking(move || {
                            let _ = join.join();
                        })
                        .await;
                    }
                    update_tray_icon(&app_guard);
                    refresh_tray_menu(&app_guard);
                    let _ = app_guard.emit("permission_required", ());
                    let _ = app_guard.emit("stay_active_state_changed", serde_json::json!({ "active": false }));
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    window.hide().unwrap();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
