use std::fs;
use std::path::PathBuf;
use tauri::State;
use crate::config::ConfigState;

pub fn get_drafts_dir_path(state: &State<ConfigState>) -> Result<PathBuf, String> {
    let config = state.0.lock().unwrap();
    let path = PathBuf::from(&config.storage_path);
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

#[tauri::command]
pub fn save_draft(state: State<ConfigState>, content: String, filename: String) -> Result<(), String> {
    let mut path = get_drafts_dir_path(&state)?;
    path.push(filename);
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_draft(state: State<ConfigState>, filename: String) -> Result<String, String> {
    let mut path = get_drafts_dir_path(&state)?;
    path.push(filename);
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_draft_file(state: State<ConfigState>, filename: String) -> Result<(), String> {
    let mut path = get_drafts_dir_path(&state)?;
    path.push(filename);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_draft_modified_time(state: State<ConfigState>, filename: String) -> Result<u64, String> {
    let mut path = get_drafts_dir_path(&state)?;
    path.push(filename);
    if path.exists() {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    return Ok(duration.as_millis() as u64);
                }
            }
        }
    }
    Ok(0)
}

#[tauri::command]
pub fn save_export_text(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_export_binary(path: String, content: Vec<u8>) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

