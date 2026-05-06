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

pub fn get_draft_attachments_dir(state: &State<ConfigState>, draft_id: &str) -> Result<PathBuf, String> {
    let mut path = get_drafts_dir_path(state)?;
    path.push("attachments");
    path.push(draft_id);
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

#[tauri::command]
pub fn get_draft_files(state: State<ConfigState>, draft_id: String) -> Result<Vec<String>, String> {
    let dir = get_draft_attachments_dir(&state, &draft_id)?;
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            // ignore invisible files like .DS_Store
                            if !name.starts_with(".") {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(files)
}

#[tauri::command]
pub fn save_file_attachment(state: State<ConfigState>, draft_id: String, file_name: String, content: Vec<u8>) -> Result<String, String> {
    let mut dest_dir = get_draft_attachments_dir(&state, &draft_id)?;
    dest_dir.push(&file_name);
    fs::write(&dest_dir, content).map_err(|e| e.to_string())?;
    Ok(file_name)
}

#[tauri::command]
pub fn copy_file_to_draft(state: State<ConfigState>, draft_id: String, source_path: String) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() || !source.is_file() {
        return Err("Source file does not exist".to_string());
    }
    let file_name = source.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
        
    let mut dest_dir = get_draft_attachments_dir(&state, &draft_id)?;
    dest_dir.push(file_name);
    
    fs::copy(&source, &dest_dir).map_err(|e| e.to_string())?;
    Ok(file_name.to_string())
}

#[tauri::command]
pub fn delete_draft_file_attachment(state: State<ConfigState>, draft_id: String, file_name: String) -> Result<(), String> {
    let mut path = get_draft_attachments_dir(&state, &draft_id)?;
    path.push(file_name);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_draft_file(state: State<ConfigState>, draft_id: String, file_name: String) -> Result<(), String> {
    let mut path = get_draft_attachments_dir(&state, &draft_id)?;
    path.push(file_name);
    open::that(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_draft_attachments_dir(state: State<ConfigState>, draft_id: String) -> Result<(), String> {
    let path = get_draft_attachments_dir(&state, &draft_id)?;
    open::that(&path).map_err(|e| e.to_string())
}

