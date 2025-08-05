use tauri::Manager;

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
            let _ = SetWindowDisplayAffinity(HWND(hwnd.0 as *mut core::ffi::c_void), WDA_MONITOR);
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let _ = app.remove_menu();
            if let Some(window) = app.get_webview_window("main") {
                block_capture(&window);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.eval(
                    r#"document.addEventListener('contextmenu', e => e.preventDefault());"#,
                );
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
