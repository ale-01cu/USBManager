use crate::db::{Database, FileSnapshot};
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const DEBOUNCE_DURATION: Duration = Duration::from_secs(3);

pub struct FileWatcher;

impl FileWatcher {
    pub fn watch_mount(
        mount_point: String,
        activity_id: i64,
        db: Arc<Database>,
        app_handle: AppHandle,
    ) -> notify::Result<notify::RecommendedWatcher> {
        let mount_path = mount_point.clone();
        let recent_files = Arc::new(Mutex::new(HashMap::new()));

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if event.kind.is_create() || event.kind.is_modify() {
                        for path in event.paths {
                            if path.is_file() {
                                Self::handle_copy_event(
                                    &path,
                                    &mount_path,
                                    activity_id,
                                    &db,
                                    &app_handle,
                                    Arc::clone(&recent_files),
                                );
                            }
                        }
                    }
                }
                Err(e) => println!("[Watcher] Error: {:?}", e),
            })?;

        watcher.watch(Path::new(&mount_point), RecursiveMode::Recursive)?;
        println!("[Watcher] Iniciado en: {}", mount_point);

        Ok(watcher)
    }

    fn handle_copy_event(
        path: &Path,
        _mount_point: &str,
        activity_id: i64,
        db: &Arc<Database>,
        app_handle: &AppHandle,
        recent_files: Arc<Mutex<HashMap<String, Instant>>>,
    ) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_path = path.to_string_lossy().to_string();

        if file_name.starts_with('~') || file_name.starts_with('.') {
            return;
        }

        let mut recent = recent_files.lock().unwrap();

        if let Some(&last_seen) = recent.get(&file_path) {
            if last_seen.elapsed() < DEBOUNCE_DURATION {
                return;
            }
        }

        recent.insert(file_path.clone(), Instant::now());
        drop(recent);

        let metadata = std::fs::metadata(path).ok();
        let size = metadata.map(|m| m.len() as i64).unwrap_or(0);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        let snapshot = FileSnapshot {
            id: None,
            activity_log_id: activity_id,
            file_path: file_path.clone(),
            file_name: file_name.clone(),
            file_extension: extension,
            file_size: size,
            is_folder: false,
        };

        if let Ok(_) = db.insert_file_snapshot(&snapshot) {
            let _ = app_handle.emit(
                "file-copy-detected",
                serde_json::json!({
                    "activity_id": activity_id,
                    "file_name": file_name,
                    "file_size": size,
                    "path": file_path
                }),
            );
        }
    }
}
