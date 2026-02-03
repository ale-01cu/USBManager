use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub serial_number: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub total_capacity: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityLog {
    pub id: i64,
    pub device_id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum EventType {
    Connect,
    Disconnect,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Connect => "CONNECT",
            EventType::Disconnect => "DISCONNECT",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileSnapshot {
    pub id: Option<i64>,
    pub activity_log_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: Option<String>,
    pub file_size: i64,
    pub is_folder: bool,
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        let db_path = app_data_dir.join("usb_manager.db");
        println!("[DB] Initializing database at: {:?}", db_path);

        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_tables()?;
        println!("[DB] Database initialized successfully");

        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Tabla devices
        conn.execute(
            "CREATE TABLE IF NOT EXISTS devices (
                serial_number TEXT PRIMARY KEY,
                vendor_id INTEGER NOT NULL,
                product_id INTEGER NOT NULL,
                name TEXT,
                manufacturer TEXT,
                total_capacity INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Tabla activity_log
        conn.execute(
            "CREATE TABLE IF NOT EXISTS activity_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                event_type TEXT NOT NULL CHECK(event_type IN ('CONNECT', 'DISCONNECT')),
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (device_id) REFERENCES devices(serial_number)
            )",
            [],
        )?;

        // Tabla file_snapshots
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                activity_log_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_extension TEXT,
                file_size INTEGER NOT NULL,
                is_folder BOOLEAN NOT NULL DEFAULT 0,
                scanned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (activity_log_id) REFERENCES activity_log(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Índices para búsquedas más rápidas
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_activity_log_device_id ON activity_log(device_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_activity_log_timestamp ON activity_log(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_snapshots_activity_id ON file_snapshots(activity_log_id)",
            [],
        )?;

        Ok(())
    }

    // Upsert device (insertar o actualizar)
    pub fn upsert_device(&self, device: &Device) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO devices (serial_number, vendor_id, product_id, name, manufacturer, total_capacity, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
             ON CONFLICT(serial_number) DO UPDATE SET
                vendor_id = excluded.vendor_id,
                product_id = excluded.product_id,
                name = excluded.name,
                manufacturer = excluded.manufacturer,
                total_capacity = excluded.total_capacity,
                updated_at = CURRENT_TIMESTAMP",
            params![
                device.serial_number,
                device.vendor_id,
                device.product_id,
                device.name,
                device.manufacturer,
                device.total_capacity,
            ],
        )?;

        println!("[DB] Device upserted: {}", device.serial_number);
        Ok(())
    }

    // Crear registro de actividad
    pub fn create_activity_log(&self, device_id: &str, event_type: EventType) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO activity_log (device_id, event_type, timestamp)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![device_id, event_type.as_str()],
        )?;

        let id = conn.last_insert_rowid();
        println!(
            "[DB] Activity log created: id={}, device={}, type={}",
            id,
            device_id,
            event_type.as_str()
        );

        Ok(id)
    }

    // Insertar snapshot de archivo en batch (más eficiente)
    pub fn insert_file_snapshots_batch(&self, snapshots: &[FileSnapshot]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO file_snapshots (activity_log_id, file_path, file_name, file_extension, file_size, is_folder)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for snapshot in snapshots {
                stmt.execute(params![
                    snapshot.activity_log_id,
                    snapshot.file_path,
                    snapshot.file_name,
                    snapshot.file_extension,
                    snapshot.file_size,
                    snapshot.is_folder,
                ])?;
            }
        }

        tx.commit()?;
        println!("[DB] Inserted {} file snapshots", snapshots.len());

        Ok(())
    }

    // Insertar snapshot individual
    pub fn insert_file_snapshot(&self, snapshot: &FileSnapshot) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO file_snapshots (activity_log_id, file_path, file_name, file_extension, file_size, is_folder)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                snapshot.activity_log_id,
                snapshot.file_path,
                snapshot.file_name,
                snapshot.file_extension,
                snapshot.file_size,
                snapshot.is_folder,
            ],
        )?;

        Ok(())
    }

    // Obtener historial de actividad
    pub fn get_activity_history(&self, limit: i64) -> Result<Vec<ActivityLog>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, device_id, event_type, timestamp 
             FROM activity_log 
             ORDER BY timestamp DESC 
             LIMIT ?1",
        )?;

        let activity_iter = stmt.query_map(params![limit], |row| {
            let event_type_str: String = row.get(2)?;
            let event_type = match event_type_str.as_str() {
                "CONNECT" => EventType::Connect,
                "DISCONNECT" => EventType::Disconnect,
                _ => EventType::Connect, // default
            };

            Ok(ActivityLog {
                id: row.get(0)?,
                device_id: row.get(1)?,
                event_type,
                timestamp: row.get(3)?,
            })
        })?;

        let mut activities = Vec::new();
        for activity in activity_iter {
            activities.push(activity?);
        }

        Ok(activities)
    }

    // Obtener snapshots de un activity_log específico
    pub fn get_file_snapshots(&self, activity_log_id: i64) -> Result<Vec<FileSnapshot>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, activity_log_id, file_path, file_name, file_extension, file_size, is_folder
             FROM file_snapshots 
             WHERE activity_log_id = ?1
             ORDER BY file_path",
        )?;

        let snapshot_iter = stmt.query_map(params![activity_log_id], |row| {
            Ok(FileSnapshot {
                id: row.get(0)?,
                activity_log_id: row.get(1)?,
                file_path: row.get(2)?,
                file_name: row.get(3)?,
                file_extension: row.get(4)?,
                file_size: row.get(5)?,
                is_folder: row.get(6)?,
            })
        })?;

        let mut snapshots = Vec::new();
        for snapshot in snapshot_iter {
            snapshots.push(snapshot?);
        }

        Ok(snapshots)
    }

    // Obtener dispositivos registrados
    pub fn get_devices(&self) -> Result<Vec<Device>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT serial_number, vendor_id, product_id, name, manufacturer, total_capacity
             FROM devices
             ORDER BY updated_at DESC",
        )?;

        let device_iter = stmt.query_map([], |row| {
            Ok(Device {
                serial_number: row.get(0)?,
                vendor_id: row.get(1)?,
                product_id: row.get(2)?,
                name: row.get(3)?,
                manufacturer: row.get(4)?,
                total_capacity: row.get(5)?,
            })
        })?;

        let mut devices = Vec::new();
        for device in device_iter {
            devices.push(device?);
        }

        Ok(devices)
    }

    // Obtener estadísticas de un escaneo
    pub fn get_scan_stats(&self, activity_log_id: i64) -> Result<(i64, i64)> {
        let conn = self.conn.lock().unwrap();

        let total_files: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_snapshots WHERE activity_log_id = ?1 AND is_folder = 0",
            params![activity_log_id],
            |row| row.get(0),
        )?;

        let total_folders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_snapshots WHERE activity_log_id = ?1 AND is_folder = 1",
            params![activity_log_id],
            |row| row.get(0),
        )?;

        Ok((total_files, total_folders))
    }

    // Obtener snapshots del último CONNECT de un dispositivo específico
    pub fn get_latest_device_snapshots(&self, device_id: &str) -> Result<(i64, Vec<FileSnapshot>)> {
        let conn = self.conn.lock().unwrap();

        // Obtener el último activity_log CONNECT para este dispositivo
        let activity_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM activity_log 
             WHERE device_id = ?1 AND event_type = 'CONNECT'
             ORDER BY timestamp DESC
             LIMIT 1",
                params![device_id],
                |row| row.get(0),
            )
            .ok();

        match activity_id {
            Some(id) => {
                let mut stmt = conn.prepare(
                    "SELECT id, activity_log_id, file_path, file_name, file_extension, file_size, is_folder
                     FROM file_snapshots 
                     WHERE activity_log_id = ?1
                     ORDER BY file_path"
                )?;

                let snapshot_iter = stmt.query_map(params![id], |row| {
                    Ok(FileSnapshot {
                        id: row.get(0)?,
                        activity_log_id: row.get(1)?,
                        file_path: row.get(2)?,
                        file_name: row.get(3)?,
                        file_extension: row.get(4)?,
                        file_size: row.get(5)?,
                        is_folder: row.get(6)?,
                    })
                })?;

                let mut snapshots = Vec::new();
                for snapshot in snapshot_iter {
                    snapshots.push(snapshot?);
                }

                println!(
                    "[DB] Found {} snapshots for device {} (activity_id: {})",
                    snapshots.len(),
                    device_id,
                    id
                );
                Ok((id, snapshots))
            }
            None => {
                println!("[DB] No CONNECT activity found for device {}", device_id);
                Ok((0, Vec::new()))
            }
        }
    }

    // Obtener todos los snapshots de un dispositivo (de todos sus connections)
    pub fn get_all_device_snapshots(
        &self,
        device_id: &str,
    ) -> Result<Vec<(i64, String, Vec<FileSnapshot>)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT al.id, al.timestamp 
             FROM activity_log al
             WHERE al.device_id = ?1 AND al.event_type = 'CONNECT'
             ORDER BY al.timestamp DESC",
        )?;

        let activity_iter = stmt.query_map(params![device_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut results = Vec::new();
        for activity_result in activity_iter {
            let (activity_id, timestamp) = activity_result?;

            let mut snapshot_stmt = conn.prepare(
                "SELECT id, activity_log_id, file_path, file_name, file_extension, file_size, is_folder
                 FROM file_snapshots 
                 WHERE activity_log_id = ?1
                 ORDER BY file_path"
            )?;

            let snapshot_iter = snapshot_stmt.query_map(params![activity_id], |row| {
                Ok(FileSnapshot {
                    id: row.get(0)?,
                    activity_log_id: row.get(1)?,
                    file_path: row.get(2)?,
                    file_name: row.get(3)?,
                    file_extension: row.get(4)?,
                    file_size: row.get(5)?,
                    is_folder: row.get(6)?,
                })
            })?;

            let mut snapshots = Vec::new();
            for snapshot in snapshot_iter {
                snapshots.push(snapshot?);
            }

            results.push((activity_id, timestamp, snapshots));
        }

        println!(
            "[DB] Found {} connection events for device {}",
            results.len(),
            device_id
        );
        Ok(results)
    }
}

// Singleton para acceso global
use std::sync::OnceLock;

static DB_INSTANCE: OnceLock<Arc<Database>> = OnceLock::new();

pub fn init_database(app_data_dir: PathBuf) -> Result<Arc<Database>> {
    let db = Arc::new(Database::new(app_data_dir)?);
    let _ = DB_INSTANCE.set(db.clone());
    Ok(db)
}

pub fn get_database() -> Option<Arc<Database>> {
    DB_INSTANCE.get().cloned()
}
