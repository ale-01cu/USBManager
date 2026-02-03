use rusb::{Context, Device, DeviceList, UsbContext};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use sysinfo::Disks;
use crate::db::{Database, Device as DbDevice, EventType, get_database};
use crate::file_scanner::FileScanner;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UsbDevice {
    pub id: u16,
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
    pub device_mount_map: Arc<Mutex<HashMap<String, String>>>, // serial -> mount_point
}

impl UsbMonitor {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(Vec::new())),
            app_handle: None,
            db: get_database(),
            device_mount_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn set_db(&mut self, db: Arc<Database>) {
        self.db = Some(db);
    }

    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    fn get_device_info<T: UsbContext>(device: &Device<T>) -> Result<UsbDevice, Box<dyn std::error::Error>> {
        let device_desc = device.device_descriptor()?;
        
        let mut product_name = None;
        let mut manufacturer_name = None;
        let mut serial_number = None;

        // Try to open device and read string descriptors
        if let Ok(handle) = device.open() {
            if let Ok(langs) = handle.read_languages(Duration::from_secs(1)) {
                if let Some(lang) = langs.first() {
                    let lang_id = *lang;
                    if let Some(index) = device_desc.product_string_index() {
                        if let Ok(product) = handle.read_string_descriptor(lang_id, index, Duration::from_secs(1)) {
                            product_name = Some(product);
                        }
                    }
                    if let Some(index) = device_desc.manufacturer_string_index() {
                        if let Ok(manufacturer) = handle.read_string_descriptor(lang_id, index, Duration::from_secs(1)) {
                            manufacturer_name = Some(manufacturer);
                        }
                    }
                    if let Some(index) = device_desc.serial_number_string_index() {
                        if let Ok(serial) = handle.read_string_descriptor(lang_id, index, Duration::from_secs(1)) {
                            serial_number = Some(serial);
                        }
                    }
                }
            }
        }

        Ok(UsbDevice {
            id: device.address() as u16,
            vendor_id: device_desc.vendor_id(),
            product_id: device_desc.product_id(),
            product_name,
            manufacturer_name,
            serial_number,
            mount_point: None,
            total_space: None,
        })
    }

    pub fn scan_devices(&self) -> Vec<UsbDevice> {
        println!("[USB] Scanning for USB devices...");
        
        let context = match Context::new() {
            Ok(ctx) => ctx,
            Err(e) => {
                println!("[USB] Error creating USB context: {}", e);
                return Vec::new();
            }
        };
        
        let device_list = match DeviceList::new_with_context(context) {
            Ok(list) => list,
            Err(e) => {
                println!("[USB] Error getting device list: {}", e);
                return Vec::new();
            }
        };
        
        println!("[USB] Found {} raw USB devices", device_list.len());
        
        // Obtener información de discos usando sysinfo
        let disks = Disks::new_with_refreshed_list();
        
        let mut current_devices = Vec::new();
        for device in device_list.iter() {
            if let Ok(mut device_info) = Self::get_device_info(&device) {
                // Buscar punto de montante correlacionando por número de serie
                if let Some(ref serial) = device_info.serial_number {
                    for disk in &disks {
                        // En Windows, intentamos correlacionar por diferentes métodos
                        let disk_name = disk.name().to_string_lossy().to_string();
                        let mount_point = disk.mount_point().to_string_lossy().to_string();
                        
                        // Correlación simple: si el serial del USB contiene parte del nombre del disco
                        // o viceversa. También consideramos discos que no son HDD del sistema.
                        let matches = 
                            serial.to_lowercase().contains(&disk_name.to_lowercase()) ||
                            disk_name.to_lowercase().contains(&serial.to_lowercase()) ||
                            disk_name.is_empty() || // Algunos USB no tienen nombre en sysinfo
                            mount_point.to_lowercase().contains("removable") ||
                            mount_point.to_lowercase().contains("usb");
                        
                        if matches {
                            device_info.mount_point = Some(mount_point.clone());
                            device_info.total_space = Some(disk.total_space());
                            println!("[USB] Found mount point for device {}: {} ({} bytes)", 
                                serial, mount_point, disk.total_space());
                            break;
                        }
                    }
                }
                
                println!("[USB] Device: VID={:04X}, PID={:04X}, Address={}, Mount={:?}", 
                    device_info.vendor_id, device_info.product_id, device_info.id, device_info.mount_point);
                current_devices.push(device_info);
            }
        }
        
        println!("[USB] Successfully parsed {} devices", current_devices.len());
        current_devices
    }

    fn check_changes(&self) -> (Vec<UsbDevice>, Vec<UsbDevice>) {
        let current_devices = self.scan_devices();
        let previous_devices = self.devices.lock().unwrap().clone();
        
        println!("[USB] Checking changes: previous={}, current={}", 
            previous_devices.len(), current_devices.len());
        
        let mut connected_devices = Vec::new();
        let mut disconnected_devices = Vec::new();
        
        // Detectar nuevos dispositivos
        for device in &current_devices {
            let is_new = !previous_devices.iter().any(|d| {
                d.vendor_id == device.vendor_id && 
                d.product_id == device.product_id &&
                d.serial_number == device.serial_number
            });
            
            if is_new {
                connected_devices.push(device.clone());
            }
        }
        
        // Detectar dispositivos desconectados
        for device in &previous_devices {
            let still_connected = current_devices.iter().any(|d| {
                d.vendor_id == device.vendor_id && 
                d.product_id == device.product_id &&
                d.serial_number == device.serial_number
            });
            
            if !still_connected {
                disconnected_devices.push(device.clone());
            }
        }
        
        *self.devices.lock().unwrap() = current_devices;
        
        (connected_devices, disconnected_devices)
    }

    fn handle_device_connected(&self, device: &UsbDevice) {
        println!("[USB] Handling device connection: VID={:04X}, PID={:04X}", 
            device.vendor_id, device.product_id);
        
        // Generar ID único para el dispositivo
        let device_id = device.serial_number.clone().unwrap_or_else(|| {
            format!("VID{:04X}_PID{:04X}_ADDR{}", device.vendor_id, device.product_id, device.id)
        });
        
        // Guardar en base de datos
        if let Some(ref db) = self.db {
            // Crear/actualizar dispositivo
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
            
            // Crear registro de actividad
            match db.create_activity_log(&device_id, EventType::Connect) {
                Ok(activity_id) => {
                    println!("[DB] Created activity log: id={}", activity_id);
                    
                    // Guardar mapeo serial -> mount_point para escaneo posterior
                    if let Some(ref mount) = device.mount_point {
                        self.device_mount_map.lock().unwrap().insert(device_id.clone(), mount.clone());
                        
                        // Escanear archivos si hay punto de montaje
                        let mount_point = mount.clone();
                        let db_clone = db.clone();
                        let app_handle_clone = self.app_handle.clone();
                        
                        tokio::spawn(async move {
                            println!("[Scanner] Starting async scan for activity_id={}", activity_id);
                            match FileScanner::scan_and_save(&mount_point, activity_id, db_clone).await {
                                Ok(stats) => {
                                    println!("[Scanner] Scan complete: {} files, {} folders, {} bytes", 
                                        stats.total_files, stats.total_folders, stats.total_size_bytes);
                                    
                                    // Emitir evento de escaneo completado
                                    if let Some(app_handle) = app_handle_clone {
                                        let _ = app_handle.emit("usb-scan-complete", serde_json::json!({
                                            "device_id": device_id,
                                            "activity_id": activity_id,
                                            "files_scanned": stats.total_files,
                                            "folders_scanned": stats.total_folders,
                                            "total_size": stats.total_size_bytes,
                                        }));
                                    }
                                }
                                Err(e) => {
                                    println!("[Scanner] Scan failed: {}", e);
                                }
                            }
                        });
                    } else {
                        println!("[USB] No mount point found, skipping file scan");
                    }
                }
                Err(e) => {
                    println!("[DB] Error creating activity log: {}", e);
                }
            }
        } else {
            println!("[USB] No database available, skipping persistence");
        }
    }

    fn handle_device_disconnected(&self, device: &UsbDevice) {
        println!("[USB] Handling device disconnection: VID={:04X}, PID={:04X}", 
            device.vendor_id, device.product_id);
        
        // Generar ID único para el dispositivo
        let device_id = device.serial_number.clone().unwrap_or_else(|| {
            format!("VID{:04X}_PID{:04X}_ADDR{}", device.vendor_id, device.product_id, device.id)
        });
        
        // Guardar en base de datos
        if let Some(ref db) = self.db {
            match db.create_activity_log(&device_id, EventType::Disconnect) {
                Ok(activity_id) => {
                    println!("[DB] Created disconnect activity log: id={}", activity_id);
                }
                Err(e) => {
                    println!("[DB] Error creating disconnect activity log: {}", e);
                }
            }
            
            // Limpiar mapeo de mount point
            self.device_mount_map.lock().unwrap().remove(&device_id);
        }
    }

    pub fn emit_events(&self) {
        let (connected, disconnected) = self.check_changes();
        
        // Procesar conexiones
        for device in &connected {
            println!("[USB] Device connected: {:?}", device.serial_number);
            self.handle_device_connected(device);
            
            // Emitir evento al frontend
            if let Some(ref app_handle) = self.app_handle {
                app_handle
                    .emit("usb-connected", &device)
                    .unwrap_or_else(|e| eprintln!("[USB] Failed to emit connected event: {}", e));
            }
        }
        
        // Procesar desconexiones
        for device in &disconnected {
            println!("[USB] Device disconnected: {:?}", device.serial_number);
            self.handle_device_disconnected(device);
            
            // Emitir evento al frontend
            if let Some(ref app_handle) = self.app_handle {
                app_handle
                    .emit("usb-disconnected", &device)
                    .unwrap_or_else(|e| eprintln!("[USB] Failed to emit disconnected event: {}", e));
            }
        }
    }

    pub async fn start_monitoring(self) {
        println!("[USB] Starting USB monitoring loop with DB integration...");
        let monitor = Arc::new(self);
        
        loop {
            monitor.emit_events();
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn get_current_devices(&self) -> Vec<UsbDevice> {
        self.devices.lock().unwrap().clone()
    }
}

#[tauri::command]
pub async fn get_connected_devices() -> Result<Vec<UsbDevice>, String> {
    println!("[USB] Command: get_connected_devices called");
    let monitor = UsbMonitor::new();
    let devices = monitor.scan_devices();
    println!("[USB] Returning {} devices", devices.len());
    Ok(devices)
}

#[tauri::command]
pub async fn start_usb_monitoring(app_handle: AppHandle) -> Result<String, String> {
    println!("[USB] Command: start_usb_monitoring called with DB integration");
    let mut monitor = UsbMonitor::new();
    monitor.set_app_handle(app_handle);
    
    let devices = monitor.scan_devices();
    println!("[USB] Initial scan found {} devices", devices.len());
    *monitor.devices.lock().unwrap() = devices;
    
    tokio::spawn(async move {
        monitor.start_monitoring().await;
    });
    
    println!("[USB] Monitoring started successfully");
    Ok("USB monitoring started".to_string())
}

// Nuevos comandos para el frontend

#[tauri::command]
pub async fn get_device_history(limit: i64) -> Result<serde_json::Value, String> {
    if let Some(ref db) = get_database() {
        match db.get_activity_history(limit) {
            Ok(history) => {
                Ok(serde_json::json!({
                    "success": true,
                    "history": history,
                }))
            }
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
            Ok(devices) => {
                Ok(serde_json::json!({
                    "success": true,
                    "devices": devices,
                }))
            }
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
                let (files, folders) = db.get_scan_stats(activity_log_id)
                    .unwrap_or((0, 0));
                
                Ok(serde_json::json!({
                    "success": true,
                    "snapshots": snapshots,
                    "stats": {
                        "total_files": files,
                        "total_folders": folders,
                    }
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
                let (files, folders) = if activity_id > 0 {
                    db.get_scan_stats(activity_id).unwrap_or((0, 0))
                } else {
                    (0, 0)
                };
                
                Ok(serde_json::json!({
                    "success": true,
                    "device_id": device_id,
                    "activity_id": activity_id,
                    "snapshots": snapshots,
                    "stats": {
                        "total_files": files,
                        "total_folders": folders,
                    }
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
                let scans: Vec<serde_json::Value> = results.into_iter().map(|(activity_id, timestamp, snapshots)| {
                    let file_count = snapshots.iter().filter(|s| !s.is_folder).count();
                    let folder_count = snapshots.iter().filter(|s| s.is_folder).count();
                    
                    serde_json::json!({
                        "activity_id": activity_id,
                        "timestamp": timestamp,
                        "snapshot_count": snapshots.len(),
                        "file_count": file_count,
                        "folder_count": folder_count,
                    })
                }).collect();
                
                Ok(serde_json::json!({
                    "success": true,
                    "device_id": device_id,
                    "scans": scans,
                }))
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

impl PartialEq for UsbDevice {
    fn eq(&self, other: &Self) -> bool {
        self.vendor_id == other.vendor_id && 
        self.product_id == other.product_id &&
        self.serial_number == other.serial_number
    }
}