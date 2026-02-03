mod usb_monitor;
mod db;
mod file_scanner;
mod file_watcher;

use std::sync::Arc;
use usb_monitor::{
    get_connected_devices, 
    start_usb_monitoring,
    get_device_history,
    get_registered_devices,
    get_file_snapshots,
    get_device_files,
    get_device_all_scans,
};
use db::init_database;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn minimize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn toggle_maximize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn close_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

#[tauri::command]
fn start_dragging(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.start_dragging();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_connected_devices, 
            start_usb_monitoring,
            get_device_history,
            get_registered_devices,
            get_file_snapshots,
            get_device_files,
            get_device_all_scans,
            minimize_window,
            toggle_maximize_window,
            close_window,
            start_dragging,
        ])
        .setup(|app| {
            println!("[App] Setting up USB Manager with persistence...");
            
            // Obtener directorio de datos de la aplicación
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            
            // Crear directorio si no existe
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir)
                    .expect("Failed to create app data directory");
            }
            
            // Inicializar base de datos
            match init_database(app_data_dir) {
                Ok(db) => {
                    println!("[App] Database initialized successfully");
                    
                    // Iniciar monitoreo USB con DB
                    let app_handle = app.handle().clone();

                    let mut monitor_to_start = usb_monitor::UsbMonitor::new();
                    monitor_to_start.set_db(db.clone());
                    monitor_to_start.set_app_handle(app_handle.clone());
                    
                    let shared_monitor = Arc::new(monitor_to_start);
                    app.manage(shared_monitor.clone());

                    tauri::async_runtime::spawn(async move {
                        // Scan inicial
                        let devices = shared_monitor.scan_devices();
                        println!("[App] Initial scan found {} devices", devices.len());
                        {
                            let mut dev_lock = shared_monitor.devices.lock().unwrap();
                            *dev_lock = devices;
                        }
                        
                        // Iniciar loop de monitoreo
                        shared_monitor.start_monitoring_shared().await;
                    });
                }
                Err(e) => {
                    eprintln!("[App] Failed to initialize database: {}", e);
                    eprintln!("[App] Continuing without persistence...");
                    
                    let app_handle = app.handle().clone();
                    let mut monitor_to_start = usb_monitor::UsbMonitor::new();
                    monitor_to_start.set_app_handle(app_handle.clone());
                    
                    let shared_monitor = Arc::new(monitor_to_start);
                    app.manage(shared_monitor.clone());

                    tauri::async_runtime::spawn(async move {
                        shared_monitor.start_monitoring_shared().await;
                    });
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
