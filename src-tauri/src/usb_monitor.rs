use rusb::{Context, Device, DeviceList};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use sysinfo::Disks;
use crate::db::{Database, Device as DbDevice, EventType, get_database};
use crate::file_scanner::FileScanner;
use crate::file_watcher::FileWatcher;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UsbDevice {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_name: Option<String>,
    pub manufacturer_name: Option<String>,
    pub serial_number: Option<String>,
    pub mount_point: Option<String>,
    pub total_space: Option<u64>,
}

pub struct UsbMonitor {
    pub devices: Arc<Mutex<Vec<UsbDevice>>>,
    pub app_handle: Option<AppHandle>,
    pub db: Option<Arc<Database>>,
    pub device_mount_map: Arc<Mutex<HashMap<String, String>>>,
    pub active_watchers: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

impl UsbMonitor {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(Vec::new())),
            app_handle: None,
            db: None,
            device_mount_map: Arc::new(Mutex::new(HashMap::new())),
            active_watchers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_db(&mut self, db: Arc<Database>) {
        self.db = Some(db);
    }

    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    fn get_rusb_details(device: &Device<Context>) -> (u16, u16, Option<String>, Option<String>, Option<String>) {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => return (0, 0, None, None, None),
        };

        let vid = device_desc.vendor_id();
        let pid = device_desc.product_id();
        let mut product = None;
        let mut manufacturer = None;
        let mut serial = None;

        if let Ok(handle) = device.open() {
            if let Ok(langs) = handle.read_languages(Duration::from_millis(200)) {
                if let Some(lang_id) = langs.first() {
                    if let Some(idx) = device_desc.product_string_index() {
                        product = handle.read_string_descriptor(*lang_id, idx, Duration::from_millis(100)).ok();
                    }
                    if let Some(idx) = device_desc.manufacturer_string_index() {
                        manufacturer = handle.read_string_descriptor(*lang_id, idx, Duration::from_millis(100)).ok();
                    }
                    if let Some(idx) = device_desc.serial_number_string_index() {
                        serial = handle.read_string_descriptor(*lang_id, idx, Duration::from_millis(100)).ok();
                    }
                }
            }
        }

        (vid, pid, product, manufacturer, serial)
    }

    pub fn scan_devices(&self) -> Vec<UsbDevice> {
        let mut final_list = Vec::new();
        
        let disks = Disks::new_with_refreshed_list();
        
        let mut rusb_devices = Vec::new();
        if let Ok(context) = Context::new() {
            if let Ok(list) = DeviceList::new_with_context(context) {
                for device in list.iter() {
                    let details = Self::get_rusb_details(&device);
                    rusb_devices.push((device, details));
                }
            }
        }

        for disk in &disks {
            if disk.is_removable() {
                let mount_point = disk.mount_point().to_string_lossy().to_string();
                let disk_name = disk.name().to_string_lossy().to_string();
                
                let mut vid = 0;
                let mut pid = 0;
                let mut product_name = if disk_name.is_empty() { "USB Drive".to_string() } else { disk_name.clone() };
                let mut manufacturer = "Generic Storage".to_string();
                let mut serial = None;

                for (_, (r_vid, r_pid, r_prod, r_man, r_serial)) in &rusb_devices {
                    let mut match_found = false;
                    
                    if let Some(s) = r_serial {
                        if !s.is_empty() && (disk_name.contains(s) || s.contains(&disk_name)) {
                            match_found = true;
                        }
                    }
                    
                    if match_found {
                        vid = *r_vid;
                        pid = *r_pid;
                        if let Some(p) = r_prod { product_name = p.clone(); }
                        if let Some(m) = r_man { manufacturer = m.clone(); }
                        serial = r_serial.clone();
                        break; 
                    }
                }

                let final_serial = serial.unwrap_or_else(|| {
                    format!("DISK_{}_{}", mount_point.replace(":", "").replace("\\", ""), disk.total_space())
                });

                final_list.push(UsbDevice {
                    id: final_serial.clone(),
                    vendor_id: vid,
                    product_id: pid,
                    product_name: Some(product_name),
                    manufacturer_name: Some(manufacturer),
                    serial_number: Some(final_serial),
                    mount_point: Some(mount_point),
                    total_space: Some(disk.total_space()),
                });
            }
        }

        println!("[USB] Scan finished. Found {} storage devices.", final_list.len());
        final_list
    }

    fn check_changes(&self) -> (Vec<UsbDevice>, Vec<UsbDevice>) {
        let current_devices = self.scan_devices();
        let previous_devices = self.devices.lock().unwrap().clone();
        
        let mut connected_devices = Vec::new();
        let mut disconnected_devices = Vec::new();

        for device in &current_devices {
            let is_new = !previous_devices.iter().any(|d| d.serial_number == device.serial_number);
            if is_new {
                connected_devices.push(device.clone());
            }
        }

        for device in &previous_devices {
            let still_connected = current_devices.iter().any(|d| d.serial_number == device.serial_number);
            if !still_connected {
                disconnected_devices.push(device.clone());
            }
        }

        *self.devices.lock().unwrap() = current_devices;
        (connected_devices, disconnected_devices)
    }

    fn handle_device_connected(&self, device: &UsbDevice) {
        let device_id = device.serial_number.clone().unwrap_or_default();

        println!("[USB] Device Logic Connected: {} (Mount: {:?})", device_id, device.mount_point);

        if let Some(ref db) = self.db {
            let db_device = DbDevice {
                serial_number: device_id.clone(),
                vendor_id: device.vendor_id,
                product_id: device.product_id,
                name: device.product_name.clone(),
                manufacturer: device.manufacturer_name.clone(),
                total_capacity: device.total_space.map(|s| s as i64),
            };

            if let Err(e) = db.upsert_device(&db_device) {
                println!("[DB] Error upserting device: {}", e);
            }

            match db.create_activity_log(&device_id, EventType::Connect) {
                Ok(activity_id) => {
                    if let Some(ref mount) = device.mount_point {
                        self.device_mount_map.lock().unwrap().insert(device_id.clone(), mount.clone());

                        let mount_point = mount.clone();
                        let db_clone = db.clone();
                        let app_handle_clone = self.app_handle.clone();
                        let dev_id_clone = device_id.clone();

                        match FileWatcher::watch_mount(
                            mount_point.clone(),
                            activity_id,
                            db_clone.clone(),
                            app_handle_clone.clone().unwrap(),
                        ) {
                            Ok(watcher) => {
                                self.active_watchers.lock().unwrap().insert(device_id.clone(), watcher);
                            }
                            Err(e) => println!("[Watcher] No se pudo iniciar: {}", e),
                        }

                        tokio::spawn(async move {
                            println!("[Scanner] Starting scan for {}", mount_point);
                            match FileScanner::scan_and_save(&mount_point, activity_id, db_clone).await {
                                Ok(stats) => {
                                    println!("[Scanner] Scan complete");
                                    if let Some(app_handle) = app_handle_clone {
                                        let _ = app_handle.emit("usb-scan-complete", serde_json::json!({
                                            "device_id": dev_id_clone,
                                            "activity_id": activity_id,
                                            "files_scanned": stats.total_files,
                                            "total_size": stats.total_size_bytes,
                                        }));
                                    }
                                }
                                Err(e) => println!("[Scanner] Error: {}", e),
                            }
                        });
                    }
                }
                Err(e) => println!("[DB] Error creating log: {}", e),
            }
        }
    }

    fn handle_device_disconnected(&self, device: &UsbDevice) {
        let device_id = device.serial_number.clone().unwrap_or_default();
        println!("[USB] Device Logic Disconnected: {}", device_id);

        if let Some(ref db) = self.db {
            let _ = db.create_activity_log(&device_id, EventType::Disconnect);
            self.device_mount_map.lock().unwrap().remove(&device_id);
        }

        self.active_watchers.lock().unwrap().remove(&device_id);
    }

    pub fn emit_events(&self) {
        let (connected, disconnected) = self.check_changes();
        
        for device in &connected {
            self.handle_device_connected(device);
            if let Some(ref app_handle) = self.app_handle {
                let _ = app_handle.emit("usb-connected", &device);
            }
        }

        for device in &disconnected {
            self.handle_device_disconnected(device);
            if let Some(ref app_handle) = self.app_handle {
                let _ = app_handle.emit("usb-disconnected", &device);
            }
        }
    }

    pub async fn start_monitoring(self) {
        println!("[USB] Monitoring service started.");
        let monitor = Arc::new(self);
        loop {
            monitor.emit_events();
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    pub async fn start_monitoring_shared(self: Arc<Self>) {
        println!("[USB] Monitoring service started (shared).");
        loop {
            self.emit_events();
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

#[tauri::command]
pub async fn get_connected_devices(
    monitor: tauri::State<'_, Arc<UsbMonitor>>
) -> Result<Vec<UsbDevice>, String> {
    let devices = monitor.devices.lock().unwrap().clone();
    Ok(devices)
}

#[tauri::command]
pub async fn start_usb_monitoring(app_handle: AppHandle) -> Result<String, String> {
    // Este comando ya no es el principal, pero lo mantenemos por compatibilidad
    // si no se usa el estado compartido
    let mut monitor = UsbMonitor::new();
    monitor.set_app_handle(app_handle);
    
    let devices = monitor.scan_devices();
    *monitor.devices.lock().unwrap() = devices;
    
    tokio::spawn(async move {
        monitor.start_monitoring().await;
    });
    
    Ok("Monitoring started".to_string())
}

#[tauri::command]
pub async fn get_device_history(limit: i64) -> Result<serde_json::Value, String> {
    if let Some(ref db) = get_database() {
        match db.get_activity_history(limit) {
            Ok(history) => Ok(serde_json::json!({ "success": true, "history": history })),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_registered_devices() -> Result<serde_json::Value, String> {
    if let Some(ref db) = get_database() {
        match db.get_devices() {
            Ok(devices) => Ok(serde_json::json!({ "success": true, "devices": devices })),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_file_snapshots(activity_log_id: i64) -> Result<serde_json::Value, String> {
     if let Some(ref db) = get_database() {
        match db.get_file_snapshots(activity_log_id) {
            Ok(snapshots) => {
                let (files, folders) = db.get_scan_stats(activity_log_id).unwrap_or((0, 0));
                Ok(serde_json::json!({
                    "success": true, 
                    "snapshots": snapshots,
                    "stats": { "total_files": files, "total_folders": folders }
                }))
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_device_files(device_id: String) -> Result<serde_json::Value, String> {
    if let Some(ref db) = get_database() {
        match db.get_latest_device_snapshots(&device_id) {
            Ok((activity_id, snapshots)) => {
                 let (files, folders) = if activity_id > 0 { db.get_scan_stats(activity_id).unwrap_or((0, 0)) } else { (0,0) };
                 Ok(serde_json::json!({
                    "success": true,
                    "device_id": device_id,
                    "activity_id": activity_id,
                    "snapshots": snapshots,
                    "stats": { "total_files": files, "total_folders": folders }
                }))
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_device_all_scans(device_id: String) -> Result<serde_json::Value, String> {
    if let Some(ref db) = get_database() {
        match db.get_all_device_snapshots(&device_id) {
            Ok(results) => {
                let scans: Vec<serde_json::Value> = results.into_iter().map(|(id, time, snaps)| {
                    serde_json::json!({
                        "activity_id": id,
                        "timestamp": time,
                        "snapshot_count": snaps.len(),
                        "file_count": snaps.iter().filter(|s| !s.is_folder).count(),
                        "folder_count": snaps.iter().filter(|s| s.is_folder).count(),
                    })
                }).collect();
                Ok(serde_json::json!({ "success": true, "device_id": device_id, "scans": scans }))
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

impl PartialEq for UsbDevice {
    fn eq(&self, other: &Self) -> bool {
        self.serial_number == other.serial_number
    }
}
