mod usb_monitor;

use usb_monitor::{get_connected_devices, start_usb_monitoring};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_connected_devices, start_usb_monitoring])
        .setup(|app| {
            println!("[App] Setting up USB monitoring...");
            // Start USB monitoring when the app starts
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match start_usb_monitoring(app_handle).await {
                    Ok(msg) => println!("[App] {}", msg),
                    Err(e) => eprintln!("[App] Failed to start USB monitoring: {}", e),
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
