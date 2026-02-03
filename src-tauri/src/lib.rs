mod usb_monitor;
mod db;
mod file_scanner;

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
                    tauri::async_runtime::spawn(async move {
                        // Crear monitor con DB
                        let mut monitor = usb_monitor::UsbMonitor::new();
                        monitor.set_db(db);
                        monitor.set_app_handle(app_handle.clone());
                        
                        // Scan inicial
                        let devices = monitor.scan_devices();
                        println!("[App] Initial scan found {} devices", devices.len());
                        *monitor.devices.lock().unwrap() = devices;
                        
                        // Iniciar loop de monitoreo
                        monitor.start_monitoring().await;
                    });
                }
                Err(e) => {
                    eprintln!("[App] Failed to initialize database: {}", e);
                    eprintln!("[App] Continuing without persistence...");
                    
                    // Fallback: iniciar sin DB
                    let app_handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        match start_usb_monitoring(app_handle).await {
                            Ok(msg) => println!("[App] {}", msg),
                            Err(e) => eprintln!("[App] Failed to start USB monitoring: {}", e),
                        }
                    });
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
