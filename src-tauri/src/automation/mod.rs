//! Stay-active loop: enigo mouse move and keepawake assertion.

#[cfg(debug_assertions)]
macro_rules! dev_log {
    ($($t:tt)*) => {{
        let _msg = format!("[StayActive:auto] {}", format!($($t)*));
        eprintln!("{}", _msg);
        crate::dev_log_write(&_msg);
    }}
}
#[cfg(not(debug_assertions))]
macro_rules! dev_log {
    ($($t:tt)*) => {}
}

use enigo::Mouse;
use std::sync::atomic::{AtomicBool, Ordering};

static RUNNING: AtomicBool = AtomicBool::new(false);
static CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn set_running(r: bool) {
    dev_log!("set_running({})", r);
    RUNNING.store(r, Ordering::SeqCst);
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn cancel() {
    CANCELLED.store(true, Ordering::SeqCst);
}

pub fn clear_cancelled() {
    CANCELLED.store(false, Ordering::SeqCst);
}

/// Menu bar height (points). Avoid moving mouse into this region so we don't affect our tray icon/menu.
#[cfg(target_os = "macos")]
const MENU_BAR_AVOID_HEIGHT: i32 = 28;

/// Run the stay-active loop until cancelled (blocking). Call from std::thread::spawn.
#[allow(clippy::too_many_arguments)]
pub fn run_loop_blocking(
    interval_min_sec: u64,
    interval_max_sec: u64,
    random_interval: bool,
    move_pixels_min: u32,
    move_pixels_max: u32,
    simulate_move: bool,
    simulate_click: bool,
    click_button: String,
    prevent_sleep: bool,
) {
    dev_log!("run_loop_blocking start");
    set_running(true);
    clear_cancelled();

    #[cfg(target_os = "macos")]
    let _assertion = if prevent_sleep {
        keepawake::Builder::default()
            .display(false)
            .idle(true)
            .reason("StayActive")
            .create()
            .ok()
    } else {
        None
    };
    #[cfg(not(target_os = "macos"))]
    let _ = prevent_sleep;

    let mut enigo = if simulate_move || simulate_click {
        match enigo::Enigo::new(&enigo::Settings::default()) {
            Ok(e) => Some(e),
            Err(_) => {
                dev_log!("run_loop_blocking Enigo::new failed, setting running=false");
                set_running(false);
                return;
            }
        }
    } else {
        None
    };

    /// Optional FR-008: skip simulated move if user was active recently (e.g. last 60s).
    const USER_IDLE_THRESHOLD_SEC: f64 = 60.0;

    while !CANCELLED.load(Ordering::SeqCst) {
        let delay_secs = if random_interval {
            interval_min_sec + (rand_simple() % (interval_max_sec.saturating_sub(interval_min_sec) + 1))
        } else {
            interval_min_sec
        };
        std::thread::sleep(std::time::Duration::from_secs(delay_secs));
        if CANCELLED.load(Ordering::SeqCst) {
            break;
        }
        if seconds_since_last_user_input() < USER_IDLE_THRESHOLD_SEC {
            continue;
        }
        if let Some(enigo) = enigo.as_mut() {
            if simulate_move {
                let dx = if move_pixels_max > move_pixels_min {
                    move_pixels_min as i32
                        + (rand_simple() as i32 % (move_pixels_max - move_pixels_min + 1) as i32)
                } else {
                    move_pixels_min as i32
                };
                let dy = if move_pixels_max > move_pixels_min {
                    move_pixels_min as i32
                        + (rand_simple() as i32 % (move_pixels_max - move_pixels_min + 1) as i32)
                } else {
                    move_pixels_min as i32
                };
                #[cfg(target_os = "macos")]
                if would_move_into_menu_bar(dx, dy) {
                    // Skip moves that would open the tray/menu bar.
                } else {
                    let _ = enigo.move_mouse(dx, dy, enigo::Coordinate::Rel);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = enigo.move_mouse(dx, dy, enigo::Coordinate::Rel);
                }
            }

            if simulate_click {
                #[cfg(target_os = "macos")]
                if would_move_into_menu_bar(0, 0) {
                    // Avoid clicking in the menu bar (could open/close tray menu).
                    continue;
                }

                let button = match click_button.as_str() {
                    "right" => enigo::Button::Right,
                    _ => enigo::Button::Left,
                };
                let _ = enigo.button(button, enigo::Direction::Click);
            }
        }
    }

    dev_log!("run_loop_blocking exit (cancelled)");
    set_running(false);
}

/// On macOS, CoreGraphics uses bottom-left origin (y increases upward). Menu bar is at the top.
#[cfg(target_os = "macos")]
fn would_move_into_menu_bar(_dx: i32, dy: i32) -> bool {
    use mouse_position::mouse_position::Mouse;
    let pos = Mouse::get_mouse_position();
    let (_, y) = match pos {
        Mouse::Position { x, y } => (x, y),
        Mouse::Error => return false,
    };
    let new_y = y.saturating_add(dy);
    let top = match main_display_top_y() {
        Some(t) => t,
        None => return false,
    };
    new_y > (top - MENU_BAR_AVOID_HEIGHT)
}

#[cfg(target_os = "macos")]
fn main_display_top_y() -> Option<i32> {
    unsafe {
        let lib = libc::dlopen(
            b"/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics\0".as_ptr() as *const _,
            libc::RTLD_NOW,
        );
        if lib.is_null() {
            return None;
        }
        let main_display = libc::dlsym(lib, b"CGMainDisplayID\0".as_ptr() as *const _);
        let bounds_fn = libc::dlsym(lib, b"CGDisplayBounds\0".as_ptr() as *const _);
        if main_display.is_null() || bounds_fn.is_null() {
            libc::dlclose(lib);
            return None;
        }
        type MainDisplayFn = extern "C" fn() -> u32;
        type BoundsFn = extern "C" fn(u32) -> CGRect;
        #[repr(C)]
        struct CGRect {
            x: libc::c_double,
            y: libc::c_double,
            w: libc::c_double,
            h: libc::c_double,
        }
        let main_display_fn: MainDisplayFn = std::mem::transmute(main_display);
        let bounds_fn_typed: BoundsFn = std::mem::transmute(bounds_fn);
        let main_id = main_display_fn();
        let bounds = bounds_fn_typed(main_id);
        libc::dlclose(lib);
        Some((bounds.y + bounds.h) as i32)
    }
}

fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        % 1000
}

/// Seconds since last user input (mouse/keyboard). Returns a large value if unknown or unsupported.
#[cfg(target_os = "macos")]
fn seconds_since_last_user_input() -> f64 {
    unsafe {
        let name = std::ffi::CString::new("CGEventSourceSecondsSinceLastEventType").unwrap();
        let lib = libc::dlopen(
            b"/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics\0"
                .as_ptr() as *const _,
            libc::RTLD_NOW,
        );
        if lib.is_null() {
            return 999.0;
        }
        let sym = libc::dlsym(lib, name.as_ptr());
        if sym.is_null() {
            libc::dlclose(lib);
            return 999.0;
        }
        type Fn = extern "C" fn(u32, u32) -> libc::c_double;
        let f: Fn = std::mem::transmute(sym);
        let secs = f(0, 0);
        libc::dlclose(lib);
        if secs < 0.0 || secs.is_nan() {
            999.0
        } else {
            secs
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn seconds_since_last_user_input() -> f64 {
    999.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_running_toggles_is_running() {
        set_running(false);
        assert!(!is_running());
        set_running(true);
        assert!(is_running());
        set_running(false);
        assert!(!is_running());
    }

    #[test]
    fn cancel_and_clear_cancelled_affect_loop() {
        clear_cancelled();
        cancel();
        clear_cancelled();
    }

    #[test]
    fn start_then_stop_leaves_running_false() {
        set_running(false);
        assert!(!is_running());
        set_running(true);
        assert!(is_running());
        set_running(false);
        assert!(!is_running());
    }

    /// Documents contract: UI relies on is_running() matching actual state.
    /// stop_stay_active (commands) must call set_running(false) so tray/menu stay in sync.
    #[test]
    fn running_state_is_single_source_of_truth() {
        set_running(false);
        assert!(!is_running(), "initial state must be not running");
        set_running(true);
        assert!(is_running(), "after start, is_running() must be true for tray/menu");
        set_running(false);
        assert!(!is_running(), "after stop, is_running() must be false for tray/menu");
    }
}
