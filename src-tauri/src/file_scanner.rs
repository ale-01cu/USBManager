use std::path::Path;
use walkdir::WalkDir;
use crate::db::{FileSnapshot, Database};
use std::sync::Arc;

pub struct FileScanner;

impl FileScanner {
    /// Escanear un directorio recursivamente y devolver los snapshots
    pub fn scan_directory(mount_point: &str, activity_log_id: i64) -> Vec<FileSnapshot> {
        let mut snapshots = Vec::new();
        let mount_path = Path::new(mount_point);
        
        if !mount_path.exists() {
            println!("[Scanner] Mount point does not exist: {}", mount_point);
            return snapshots;
        }
        
        if !mount_path.is_dir() {
            println!("[Scanner] Mount point is not a directory: {}", mount_point);
            return snapshots;
        }
        
        println!("[Scanner] Starting scan of: {}", mount_point);
        
        let walker = WalkDir::new(mount_point)
            .follow_links(false)
            .max_open(100)
            .into_iter();
        
        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(e) => {
                            println!("[Scanner] Error reading metadata for {:?}: {}", path, e);
                            continue;
                        }
                    };
                    
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let file_path = path.to_string_lossy().to_string();
                    let is_folder = metadata.is_dir();
                    let file_size = if is_folder { 0 } else { metadata.len() as i64 };
                    let file_extension = path.extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_lowercase());
                    
                    snapshots.push(FileSnapshot {
                        id: None,
                        activity_log_id,
                        file_path,
                        file_name,
                        file_extension,
                        file_size,
                        is_folder,
                    });
                }
                Err(e) => {
                    println!("[Scanner] Error accessing entry: {}", e);
                }
            }
        }
        
        println!("[Scanner] Scan complete. Found {} items", snapshots.len());
        snapshots
    }
    
    /// Escanear y guardar directamente en la base de datos en batch
    pub async fn scan_and_save(
        mount_point: &str, 
        activity_log_id: i64, 
        db: Arc<Database>
    ) -> Result<ScanResult, String> {
        let snapshots = Self::scan_directory(mount_point, activity_log_id);
        let total_items = snapshots.len();
        
        if total_items == 0 {
            return Ok(ScanResult {
                total_files: 0,
                total_folders: 0,
                total_size_bytes: 0,
            });
        }
        
        // Calcular estadísticas
        let total_files = snapshots.iter().filter(|s| !s.is_folder).count();
        let total_folders = snapshots.iter().filter(|s| s.is_folder).count();
        let total_size_bytes: i64 = snapshots.iter().map(|s| s.file_size).sum();
        
        // Guardar en batch para mejor rendimiento
        match db.insert_file_snapshots_batch(&snapshots) {
            Ok(_) => {
                println!("[Scanner] Saved {} items to database", total_items);
                Ok(ScanResult {
                    total_files,
                    total_folders,
                    total_size_bytes,
                })
            }
            Err(e) => {
                let msg = format!("Failed to save snapshots: {}", e);
                println!("[Scanner] {}", msg);
                Err(msg)
            }
        }
    }
    
    /// Obtener el tamaño total de un directorio sin guardar en DB
    pub fn get_directory_size(mount_point: &str) -> u64 {
        let mut total_size = 0u64;
        
        let walker = WalkDir::new(mount_point)
            .follow_links(false)
            .into_iter();
        
        for entry in walker {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }
        
        total_size
    }
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub total_files: usize,
    pub total_folders: usize,
    pub total_size_bytes: i64,
}
