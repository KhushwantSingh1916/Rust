use tauri::Manager;
use std::process::{Command, Stdio};
use std::thread;
use winit::event_loop::EventLoopBuilder;
use winit::platform::windows::EventLoopBuilderExtWindows;

#[cfg(target_os = "windows")]
fn blackout_screen() {
    use winit::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Fullscreen, WindowBuilder},
    };

    std::thread::spawn(|| {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Blackout")
            .with_decorations(false)
            .with_resizable(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap();

        window.set_cursor_visible(false);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *control_flow = ControlFlow::Exit;
            }
        });
    });
}

#[cfg(target_os = "macos")]
fn blackout_screen() {
    // For macOS, similar concept, but use Cocoa to overlay a black NSWindow
    use cocoa::appkit::{NSApp, NSColor, NSWindow, NSBackingStoreBuffered};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSRect, NSSize, NSUInteger};

    std::thread::spawn(|| unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();
        let screen_frame = NSScreen::mainScreen(nil).frame();
        let window: id = NSWindow::alloc(nil)
            .initWithContentRect_styleMask_backing_defer_(
                screen_frame,
                1 << 14, // borderless window
                NSBackingStoreBuffered,
                false,
            )
            .autorelease();
        window.setBackgroundColor_(NSColor::blackColor(nil));
        window.makeKeyAndOrderFront_(nil);
        app.run();
    });
}

fn start_detector() {
    thread::spawn(|| {
        let mut child = Command::new("python")
            .arg("detector/detector.py") // path to YOLO script
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start Python detector");

        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            // Track if blackout window is already active
            let mut blackout_triggered = false;

            for line in reader.lines() {
                if let Ok(l) = line {
                    let lower = l.to_lowercase();

                    if (lower.contains("phone detected") || lower.contains("camera detected"))
                        && !blackout_triggered
                    {
                        println!("🔴 {} - Triggering blackout...", l.trim());
                        blackout_triggered = true;

                        // Run blackout on same thread instead of creating multiple event loops
                        #[cfg(target_os = "windows")]
                        {
                            use winit::{
                                event::{Event, WindowEvent},
                                event_loop::{ControlFlow, EventLoop},
                                window::{Fullscreen, WindowBuilder},
                            };

                            let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
                            let window = WindowBuilder::new()
                                .with_title("Blackout")
                                .with_decorations(false)
                                .with_resizable(false)
                                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                                .build(&event_loop)
                                .unwrap();

                            window.set_cursor_visible(false);
                            event_loop.run(move |event, _, control_flow| {
                                *control_flow = ControlFlow::Wait;
                                if let Event::WindowEvent {
                                    event: WindowEvent::CloseRequested,
                                    ..
                                } = event
                                {
                                    *control_flow = ControlFlow::Exit;
                                }
                            });
                        }

                        #[cfg(target_os = "macos")]
                        blackout_screen();
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
            start_detector();

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
