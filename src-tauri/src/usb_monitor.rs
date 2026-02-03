use rusb::{Context, Device, DeviceList, UsbContext};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UsbDevice {
    pub id: u16,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_name: Option<String>,
    pub manufacturer_name: Option<String>,
    pub serial_number: Option<String>,
}

pub struct UsbMonitor {
    devices: Arc<Mutex<Vec<UsbDevice>>>,
    app_handle: Option<AppHandle>,
}

impl UsbMonitor {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(Vec::new())),
            app_handle: None,
        }
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
        })
    }

    fn scan_devices(&self) -> Vec<UsbDevice> {
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
        
        let mut current_devices = Vec::new();
        for device in device_list.iter() {
            if let Ok(device_info) = Self::get_device_info(&device) {
                println!("[USB] Device: VID={:04X}, PID={:04X}, Address={}", 
                    device_info.vendor_id, device_info.product_id, device_info.id);
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
        
        for device in &current_devices {
            if !previous_devices.iter().any(|d| d.id == device.id && d.vendor_id == device.vendor_id && d.product_id == device.product_id) {
                connected_devices.push(device.clone());
            }
        }
        
        for device in &previous_devices {
            if !current_devices.iter().any(|d| d.id == device.id && d.vendor_id == device.vendor_id && d.product_id == device.product_id) {
                disconnected_devices.push(device.clone());
            }
        }
        
        *self.devices.lock().unwrap() = current_devices;
        
        (connected_devices, disconnected_devices)
    }

    pub fn emit_events(&self) {
        let (connected, disconnected) = self.check_changes();
        
        if !connected.is_empty() {
            println!("[USB] {} device(s) connected", connected.len());
        }
        if !disconnected.is_empty() {
            println!("[USB] {} device(s) disconnected", disconnected.len());
        }
        
        if let Some(app_handle) = &self.app_handle {
            for device in connected {
                println!("[USB] Emitting connected event for device: VID={:04X}, PID={:04X}", 
                    device.vendor_id, device.product_id);
                app_handle
                    .emit("usb-connected", &device)
                    .unwrap_or_else(|e| eprintln!("[USB] Failed to emit connected event: {}", e));
            }
            
            for device in disconnected {
                println!("[USB] Emitting disconnected event for device: VID={:04X}, PID={:04X}", 
                    device.vendor_id, device.product_id);
                app_handle
                    .emit("usb-disconnected", &device)
                    .unwrap_or_else(|e| eprintln!("[USB] Failed to emit disconnected event: {}", e));
            }
        } else {
            println!("[USB] Warning: No app handle set, cannot emit events");
        }
    }

    pub async fn start_monitoring(self) {
        println!("[USB] Starting USB monitoring loop...");
        loop {
            self.emit_events();
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
    println!("[USB] Command: start_usb_monitoring called");
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

impl PartialEq for UsbDevice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && 
        self.vendor_id == other.vendor_id && 
        self.product_id == other.product_id
    }
}