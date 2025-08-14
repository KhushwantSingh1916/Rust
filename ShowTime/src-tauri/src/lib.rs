use tauri::Manager;
use std::process::{Command, Stdio};
use std::thread;
use std::sync::{Arc, Mutex};

fn start_detector(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let mut child = Command::new("python")
            .arg("detector/detector.py") 
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start Python detector");

        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            let mut blackout_triggered = false;

            for line in reader.lines() {
                if let Ok(l) = line {
                    let lower = l.to_lowercase();

                    if (lower.contains("phone detected") || lower.contains("camera detected"))
                        && !blackout_triggered
                    {
                        println!("🔴 {} - Triggering blackout...", l.trim());
                        blackout_triggered = true;

                        // Block the main Tauri window instead of creating a new one
                        if let Some(window) = app_handle.get_webview_window("main") {
                            // Method 1: Hide the window content by injecting CSS
                            let _ = window.eval(
                                r#"
                                document.body.style.background = 'black';
                                document.body.innerHTML = '<div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: black; z-index: 999999;"></div>';
                                "#,
                            );

                            // Method 2: Additionally hide the window itself (optional)
                            // let _ = window.hide();
                        }
                    } else if lower.contains("no phone detected") || lower.contains("no camera detected") {
                        // Re-enable the window when threat is gone
                        if blackout_triggered {
                            println!("✅ {} - Restoring window...", l.trim());
                            blackout_triggered = false;
                            
                            if let Some(window) = app_handle.get_webview_window("main") {
                                // Reload the window to restore original content
                                let _ = window.eval("window.location.reload()");
                                // Or show the window again if it was hidden
                                // let _ = window.show();
                            }
                        }
                    }
                }
            }
        }

        let _ = child.wait();
    });
}

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

// Alternative approach: Create a Tauri command that can be called from the detector
#[tauri::command]
fn trigger_blackout(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.eval(
            r#"
            document.body.style.background = 'black';
            document.body.innerHTML = '<div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: black; z-index: 999999; display: flex; align-items: center; justify-content: center; color: white; font-size: 24px;">ACCESS BLOCKED</div>';
            "#,
        );
    }
}

#[tauri::command]
fn restore_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.eval("window.location.reload()");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if is_virtual_machine() {
        eprintln!("Blocked: Running inside a virtual machine is not allowed.");
        std::process::exit(0);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, trigger_blackout, restore_window])
        .setup(|app| {
            let _ = app.remove_menu();
            
            // Pass the app handle to the detector
            start_detector(app.handle().clone());

            if let Some(window) = app.get_webview_window("main") {
                block_capture(&window);
                
                let _ = window.eval(
                    r#"document.addEventListener('contextmenu', e => e.preventDefault());"#,
                );
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}