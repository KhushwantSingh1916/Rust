use tauri::Manager;

#[cfg(target_os = "windows")]
fn is_virtual_machine() -> bool {
    use std::process::Command;

    if let Ok(output) = Command::new("wmic")
        .args(["computersystem", "get", "model,manufacturer"])
        .output()
    {
        let data = String::from_utf8_lossy(&output.stdout).to_lowercase();
        return data.contains("virtual")
            || data.contains("vmware")
            || data.contains("vbox")
            || data.contains("qemu")
            || data.contains("kvm");
    }
    false
}

#[cfg(target_os = "macos")]
fn is_virtual_machine() -> bool {
    use std::process::Command;

    if let Ok(output) = Command::new("sysctl")
        .arg("hw.model")
        .output()
    {
        let data = String::from_utf8_lossy(&output.stdout).to_lowercase();
        return data.contains("virtual")
            || data.contains("vmware")
            || data.contains("vbox")
            || data.contains("qemu")
            || data.contains("parallels");
    }
    false
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(target_os = "windows")]
fn block_capture(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_MONITOR};

    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let _ = SetWindowDisplayAffinity(
                HWND(hwnd.0 as *mut core::ffi::c_void),
                WDA_MONITOR,
            );
        }
    }
}

#[cfg(target_os = "macos")]
fn block_capture(window: &tauri::WebviewWindow) {
    use cocoa::appkit::NSWindow;
    use cocoa::base::id;
    use cocoa::foundation::NSUInteger;

    const NSWindowSharingNone: NSUInteger = 0;
    unsafe {
        if let Some(ns_window) = window.ns_window() {
            let ns_window: id = ns_window as id;
            ns_window.setSharingType_(NSWindowSharingNone);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ✅ Detect VM before Tauri initializes and exit immediately
    if is_virtual_machine() {
        // Optional: print to log before exiting
        eprintln!("Blocked: Running inside a virtual machine is not allowed.");
        std::process::exit(0);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let _ = app.remove_menu();

            if let Some(window) = app.get_webview_window("main") {
                // Disable screen capture
                block_capture(&window);

                // Disable right click context menu
                let _ = window.eval(
                    r#"document.addEventListener('contextmenu', e => e.preventDefault());"#,
                );
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
