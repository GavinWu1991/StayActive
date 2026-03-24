//! Timer / auto-stop: preset countdown or custom end time; on expiry call stop_stay_active and emit.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// End time as unix timestamp (seconds). 0 = no active timer.
static END_TIMESTAMP_SEC: AtomicU64 = AtomicU64::new(0);
/// 0 = none, 1 = preset, 2 = custom
static MODE: AtomicU64 = AtomicU64::new(0);
/// For preset: original duration_secs (600, 1800, ...)
static DURATION_SECS: AtomicU64 = AtomicU64::new(0);
static CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn set_cancelled(c: bool) {
    CANCELLED.store(c, Ordering::SeqCst);
}

pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::SeqCst)
}

/// Set end time (unix timestamp seconds). mode: 1 = preset, 2 = custom. duration_secs for preset.
pub fn set_end(secs: u64, mode: u64, duration_secs: u64) {
    END_TIMESTAMP_SEC.store(secs, Ordering::SeqCst);
    MODE.store(mode, Ordering::SeqCst);
    DURATION_SECS.store(duration_secs, Ordering::SeqCst);
    CANCELLED.store(false, Ordering::SeqCst);
}

/// Clear timer (cancel or after expiry).
pub fn clear() {
    END_TIMESTAMP_SEC.store(0, Ordering::SeqCst);
    MODE.store(0, Ordering::SeqCst);
    DURATION_SECS.store(0, Ordering::SeqCst);
    CANCELLED.store(false, Ordering::SeqCst);
}

/// Returns (active, remaining_secs, mode, duration_secs). remaining_secs is 0 if inactive or expired.
pub fn state() -> (bool, u64, u64, u64) {
    let end = END_TIMESTAMP_SEC.load(Ordering::SeqCst);
    if end == 0 || CANCELLED.load(Ordering::SeqCst) {
        return (false, 0, 0, 0);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now >= end {
        return (false, 0, 0, 0);
    }
    let mode = MODE.load(Ordering::SeqCst);
    let duration = DURATION_SECS.load(Ordering::SeqCst);
    (true, end - now, mode, duration)
}

/// Check if timer has expired (now >= end). Caller should clear and run stop if true.
pub fn is_expired() -> bool {
    let end = END_TIMESTAMP_SEC.load(Ordering::SeqCst);
    if end == 0 || CANCELLED.load(Ordering::SeqCst) {
        return false;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now >= end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_inactive() {
        clear();
        let (active, rem, _mode, _dur) = state();
        assert!(!active);
        assert_eq!(rem, 0);
    }

    #[test]
    fn set_end_sets_active_state() {
        clear();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        set_end(now + 60, 1, 60);
        let (active, rem, mode, dur) = state();
        assert!(active);
        assert!(rem <= 60);
        assert_eq!(mode, 1);
        assert_eq!(dur, 60);
        clear();
        let (active2, _, _, _) = state();
        assert!(!active2);
    }

    #[test]
    fn cancelled_timer_is_inactive() {
        clear();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        set_end(now + 3600, 1, 3600);
        set_cancelled(true);
        let (active, rem, _mode, _dur) = state();
        assert!(!active);
        assert_eq!(rem, 0);
        clear();
    }

    #[test]
    fn is_expired_after_end_time() {
        clear();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let past = now.saturating_sub(10);
        set_end(past, 1, 1);
        assert!(is_expired());
        clear();
    }
}
