//! macOS Accessibility permission check and open System Settings.

#[cfg(all(debug_assertions, target_os = "macos"))]
macro_rules! dev_log {
    ($($t:tt)*) => {{
        let _msg = format!("[StayActive:perm] {}", format!($($t)*));
        eprintln!("{}", _msg);
        crate::dev_log_write(&_msg);
    }}
}
#[cfg(not(all(debug_assertions, target_os = "macos")))]
macro_rules! dev_log {
    ($($t:tt)*) => {}
}

#[cfg(target_os = "macos")]
fn ax_is_trusted_with_options(options: *const std::ffi::c_void) -> bool {
    use std::ffi::c_void;
    type AXIsProcessTrustedWithOptions = extern "C" fn(*const c_void) -> i32;
    unsafe {
        let name = std::ffi::CString::new("AXIsProcessTrustedWithOptions").unwrap();
        let lib = libc::dlopen(
            b"/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices\0"
                .as_ptr() as *const _,
            libc::RTLD_NOW,
        );
        if lib.is_null() {
            dev_log!("AXIsProcessTrustedWithOptions: dlopen failed");
            return false;
        }
        let sym = libc::dlsym(lib, name.as_ptr());
        if sym.is_null() {
            libc::dlclose(lib);
            dev_log!("AXIsProcessTrustedWithOptions: dlsym failed");
            return false;
        }
        let ax_check: AXIsProcessTrustedWithOptions = std::mem::transmute(sym);
        let result = ax_check(options);
        libc::dlclose(lib);
        result != 0
    }
}

#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    let trusted = ax_is_trusted_with_options(std::ptr::null());
    dev_log!("AXIsProcessTrustedWithOptions(null) => trusted={}", trusted);
    trusted
}

/// Triggers the system Accessibility permission prompt and opens System Settings.
/// Call when permission is not granted so the user can add the app. Only works when
/// the app is run as an .app bundle; when run as a raw binary (e.g. tauri dev), the
/// prompt may show the binary name and the user cannot add it via the UI.
#[cfg(target_os = "macos")]
pub fn request_accessibility_prompt() {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key, value)]);
    let trusted = ax_is_trusted_with_options(options.as_CFTypeRef());
    dev_log!("AXIsProcessTrustedWithOptions(prompt=true) => trusted={}", trusted);
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_prompt() {}

/// Opens System Settings (or System Preferences) to the Accessibility pane.
pub fn open_system_preferences_accessibility() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .output()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}
