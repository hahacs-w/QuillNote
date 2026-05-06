use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub hotkey: String,
    pub storage_path: String,
    pub line_height: f64,
    pub paragraph_spacing: f64,
    pub last_edit_hotkey: String,
    #[serde(default)]
    pub recent_storage_paths: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".draft_app_notes");
        Self {
            hotkey: "Control+Space".to_string(),
            storage_path: path.to_string_lossy().to_string(),
            line_height: 1.2,
            paragraph_spacing: 1.0, // Multiplier for default spacing
            last_edit_hotkey: "Command+Shift+BACKSPACE".to_string(),
            recent_storage_paths: Vec::new(),
        }
    }
}

pub struct ConfigState(pub Mutex<AppConfig>);

fn get_config_file_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default());
    path.push("com.wuhaopeng.app");
    let _ = fs::create_dir_all(&path);
    path.push("config.json");
    path
}

pub fn load_config() -> AppConfig {
    let path = get_config_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

#[tauri::command]
pub fn get_config(state: tauri::State<ConfigState>) -> AppConfig {
    state.0.lock().unwrap().clone()
}

use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

#[tauri::command]
pub fn save_config(
    _app: AppHandle,
    state: tauri::State<ConfigState>,
    db_state: tauri::State<crate::db_cmds::DbState>,
    mut new_config: AppConfig,
) -> Result<(), String> {
    let old_storage_path = {
        let config = state.0.lock().unwrap();
        config.storage_path.clone()
    };

    // 1. Handle storage path migration if it changed
    if new_config.storage_path != old_storage_path {
        // Update recent_storage_paths using the one from frontend as base
        let mut recents = new_config.recent_storage_paths.clone();
        
        // Remove the new path if it was already in recents
        recents.retain(|p| p != &new_config.storage_path);
        // Insert the old path at the front
        recents.insert(0, old_storage_path.clone());
        // Keep only the most recent 5
        recents.truncate(5);
        
        new_config.recent_storage_paths = recents;

        let new_path = PathBuf::from(&new_config.storage_path);
        let old_path = PathBuf::from(&old_storage_path);

        // Check if new_path is empty (or doesn't exist)
        let is_empty = if !new_path.exists() {
            true
        } else {
            fs::read_dir(&new_path)
                .map_err(|e| e.to_string())?
                .next()
                .is_none()
        };

        if is_empty {
            // Ask user if they want to move existing files
            use tauri_plugin_dialog::MessageDialogButtons;
            let move_confirmed = _app.dialog()
                .message("新的存储路径是空的。是否要将原有的草稿和数据库挪到这个位置？")
                .kind(MessageDialogKind::Info)
                .title("移动文件？")
                .buttons(MessageDialogButtons::OkCancelCustom("是的，挪过去".to_string(), "不，开启全新存储".to_string()))
                .blocking_show();

            if move_confirmed {
                // Ensure new_path exists
                if !new_path.exists() {
                    fs::create_dir_all(&new_path).map_err(|e| e.to_string())?;
                }

                // Close database connection before moving files
                {
                    let mut conn_lock = db_state.conn.lock().unwrap();
                    let dummy_conn = rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?;
                    let old_conn = std::mem::replace(&mut *conn_lock, dummy_conn);
                    drop(old_conn);
                }

                // Move files
                if old_path.exists() {
                    for entry in fs::read_dir(&old_path).map_err(|e| e.to_string())? {
                        let entry = entry.map_err(|e| e.to_string())?;
                        let path = entry.path();
                        if path.is_file() {
                            let filename = path.file_name().unwrap();
                            let mut new_file_path = new_path.clone();
                            new_file_path.push(filename);
                            if let Err(_) = fs::rename(&path, &new_file_path) {
                                fs::copy(&path, &new_file_path).map_err(|e| e.to_string())?;
                                fs::remove_file(path).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
            }
        }

        // Re-initialize DB connection in the new path
        {
            let mut conn_lock = db_state.conn.lock().unwrap();
            let dummy_conn = rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?;
            let _ = std::mem::replace(&mut *conn_lock, dummy_conn);
            
            let new_conn = crate::db_cmds::init_db(&new_config).map_err(|e| e.to_string())?;
            *conn_lock = new_conn;
        }
    }

    // 2. Save to config file
    let path = get_config_file_path();
    let content = serde_json::to_string_pretty(&new_config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;

    // 3. Update state
    let mut config = state.0.lock().unwrap();
    *config = new_config;

    Ok(())
}
