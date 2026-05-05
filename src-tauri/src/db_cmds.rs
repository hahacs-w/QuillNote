use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Mutex;
use tauri::State;
use crate::config::{AppConfig, ConfigState};

pub struct DbState {
    pub conn: Mutex<Connection>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DraftMeta {
    pub id: String,
    pub title: String,
    pub content_file: String,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
}

pub fn init_db(config: &AppConfig) -> Result<Connection, String> {
    let mut db_path = std::path::PathBuf::from(&config.storage_path);
    if !db_path.exists() {
        fs::create_dir_all(&db_path).map_err(|e| e.to_string())?;
    }
    db_path.push("drafts.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS drafts (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content_file TEXT NOT NULL,
            folder_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            parent_id TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Attempt to add parent_id to existing DBs
    let _ = conn.execute("ALTER TABLE drafts ADD COLUMN parent_id TEXT", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS draft_tags (
            draft_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (draft_id, tag_id)
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS draft_links (
            id TEXT PRIMARY KEY,
            draft_id TEXT NOT NULL,
            url_or_path TEXT NOT NULL,
            alias TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(conn)
}

// --- Folder Commands ---

#[tauri::command]
pub fn get_folders(state: State<DbState>) -> Result<Vec<Folder>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, parent_id FROM folders").map_err(|e| e.to_string())?;
    let folder_iter = stmt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder.map_err(|e| e.to_string())?);
    }
    Ok(folders)
}

#[tauri::command]
pub fn create_folder(state: State<DbState>, name: String, parent_id: Option<String>) -> Result<Folder, String> {
    let conn = state.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT INTO folders (id, name, parent_id) VALUES (?1, ?2, ?3)",
        params![id, name, parent_id],
    ).map_err(|e| e.to_string())?;

    Ok(Folder { id, name, parent_id })
}

#[tauri::command]
pub fn rename_folder(state: State<DbState>, id: String, new_name: String) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_folder(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    // In a real app, you'd want to handle orphans (drafts and subfolders).
    // For simplicity, we just delete the folder.
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}


// --- Draft Commands ---

#[tauri::command]
pub fn get_drafts(state: State<DbState>, folder_id: Option<String>, tag_id: Option<String>) -> Result<Vec<DraftMeta>, String> {
    let conn = state.conn.lock().unwrap();
    
    let mut query = "SELECT d.id, d.title, d.content_file, d.folder_id, d.created_at, d.updated_at, d.parent_id FROM drafts d".to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut where_clauses = Vec::new();

    where_clauses.push("d.parent_id IS NULL".to_string());

    if let Some(fid) = folder_id {
        where_clauses.push(format!("d.folder_id = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(fid));
    }

    if let Some(tid) = tag_id {
        query.push_str(" JOIN draft_tags dt ON d.id = dt.draft_id");
        where_clauses.push(format!("dt.tag_id = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(tid));
    }

    if !where_clauses.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&where_clauses.join(" AND "));
    }
    
    query.push_str(" ORDER BY d.updated_at DESC");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    // Create a vector of &dyn ToSql
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| &**p as &dyn rusqlite::ToSql).collect();

    let draft_iter = stmt.query_map(rusqlite::params_from_iter(params_refs), |row| {
        Ok(DraftMeta {
            id: row.get(0)?,
            title: row.get(1)?,
            content_file: row.get(2)?,
            folder_id: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            parent_id: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut drafts = Vec::new();
    for draft in draft_iter {
        drafts.push(draft.map_err(|e| e.to_string())?);
    }
    Ok(drafts)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalSearchResult {
    pub draft: DraftMeta,
    pub excerpt: String,
}

#[tauri::command]
pub fn global_search(state: State<DbState>, config_state: State<ConfigState>, query: String) -> Result<Vec<GlobalSearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, title, content_file, folder_id, created_at, updated_at, parent_id FROM drafts ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    
    let draft_iter = stmt.query_map([], |row| {
        Ok(DraftMeta {
            id: row.get(0)?,
            title: row.get(1)?,
            content_file: row.get(2)?,
            folder_id: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            parent_id: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    let drafts_dir = {
        let config = config_state.inner().0.lock().unwrap();
        std::path::PathBuf::from(&config.storage_path)
    };

    for draft_res in draft_iter {
        let draft = draft_res.map_err(|e| e.to_string())?;
        
        let mut is_match = false;
        let mut excerpt = String::new();

        // Check title match
        if draft.title.to_lowercase().contains(&query_lower) {
            is_match = true;
            excerpt = String::from("Matched in title");
        }

        // Read file content
        let mut file_path = drafts_dir.clone();
        file_path.push(&draft.content_file);
        
        if let Ok(content) = std::fs::read_to_string(file_path) {
            // Very naive HTML strip for the excerpt
            let text_content = content.replace("<p>", " ").replace("</p>", " ").replace("<br>", " ");
            
            if text_content.to_lowercase().contains(&query_lower) {
                is_match = true;
                if let Some(idx) = text_content.to_lowercase().find(&query_lower) {
                    let mut start = idx.saturating_sub(20);
                    while start > 0 && !text_content.is_char_boundary(start) {
                        start -= 1;
                    }
                    let mut end = (idx + query.len() + 20).min(text_content.len());
                    while end < text_content.len() && !text_content.is_char_boundary(end) {
                        end += 1;
                    }
                    excerpt = String::from("...");
                    excerpt.push_str(&text_content[start..end].replace('\n', " "));
                    excerpt.push_str("...");
                }
            }
        }

        if is_match {
            results.push(GlobalSearchResult {
                draft,
                excerpt,
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn create_draft(state: State<DbState>, title: String, folder_id: Option<String>) -> Result<DraftMeta, String> {
    let conn = state.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let content_file = format!("{}.html", id);
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO drafts (id, title, content_file, folder_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, title, content_file, folder_id, now, now],
    ).map_err(|e| e.to_string())?;

    Ok(DraftMeta {
        id,
        title,
        content_file,
        folder_id,
        created_at: now.clone(),
        updated_at: now,
        parent_id: None,
    })
}

#[tauri::command]
pub fn get_sub_drafts(state: State<DbState>, parent_id: String) -> Result<Vec<DraftMeta>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, title, content_file, folder_id, created_at, updated_at, parent_id FROM drafts WHERE parent_id = ?1 ORDER BY created_at ASC").map_err(|e| e.to_string())?;
    
    let draft_iter = stmt.query_map([parent_id], |row| {
        Ok(DraftMeta {
            id: row.get(0)?,
            title: row.get(1)?,
            content_file: row.get(2)?,
            folder_id: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            parent_id: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut drafts = Vec::new();
    for draft in draft_iter {
        drafts.push(draft.map_err(|e| e.to_string())?);
    }
    Ok(drafts)
}

#[tauri::command]
pub fn create_sub_draft(state: State<DbState>, title: String, parent_id: String) -> Result<DraftMeta, String> {
    let conn = state.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let content_file = format!("{}.html", id);
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO drafts (id, title, content_file, parent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, title, content_file, parent_id, now, now],
    ).map_err(|e| e.to_string())?;

    Ok(DraftMeta {
        id,
        title,
        content_file,
        folder_id: None,
        created_at: now.clone(),
        updated_at: now,
        parent_id: Some(parent_id),
    })
}

#[tauri::command]
pub fn update_draft_meta(state: State<DbState>, id: String, title: String, folder_id: Option<String>) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE drafts SET title = ?1, folder_id = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, folder_id, now, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_draft_timestamp(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE drafts SET updated_at = ?1 WHERE id = ?2",
        params![now, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_draft_meta(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute("DELETE FROM drafts WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Tag Commands ---

#[tauri::command]
pub fn get_tags(state: State<DbState>) -> Result<Vec<Tag>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name").map_err(|e| e.to_string())?;
    let tag_iter = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut tags = Vec::new();
    for tag in tag_iter {
        tags.push(tag.map_err(|e| e.to_string())?);
    }
    Ok(tags)
}

#[tauri::command]
pub fn get_draft_tags(state: State<DbState>, draft_id: String) -> Result<Vec<Tag>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("
        SELECT t.id, t.name 
        FROM tags t 
        JOIN draft_tags dt ON t.id = dt.tag_id 
        WHERE dt.draft_id = ?1
    ").map_err(|e| e.to_string())?;
    
    let tag_iter = stmt.query_map([draft_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut tags = Vec::new();
    for tag in tag_iter {
        tags.push(tag.map_err(|e| e.to_string())?);
    }
    Ok(tags)
}

#[tauri::command]
pub fn add_tag_to_draft(state: State<DbState>, draft_id: String, tag_name: String) -> Result<Tag, String> {
    let conn = state.conn.lock().unwrap();
    
    // Check if tag exists, if not create it
    let tag_id: String = match conn.query_row(
        "SELECT id FROM tags WHERE name = ?1",
        params![tag_name],
        |row| row.get(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let new_id = Uuid::new_v4().to_string();
            conn.execute("INSERT INTO tags (id, name) VALUES (?1, ?2)", params![new_id, tag_name]).map_err(|e| e.to_string())?;
            new_id
        },
        Err(e) => return Err(e.to_string()),
    };

    // Add relation if it doesn't exist
    conn.execute(
        "INSERT OR IGNORE INTO draft_tags (draft_id, tag_id) VALUES (?1, ?2)",
        params![draft_id, tag_id],
    ).map_err(|e| e.to_string())?;

    Ok(Tag { id: tag_id, name: tag_name })
}

#[tauri::command]
pub fn remove_tag_from_draft(state: State<DbState>, draft_id: String, tag_id: String) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM draft_tags WHERE draft_id = ?1 AND tag_id = ?2",
        params![draft_id, tag_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Link Commands ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DraftLink {
    pub id: String,
    pub draft_id: String,
    pub url_or_path: String,
    pub alias: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub fn get_draft_links(state: State<DbState>, draft_id: String) -> Result<Vec<DraftLink>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, draft_id, url_or_path, alias, created_at 
         FROM draft_links 
         WHERE draft_id = ?1 OR draft_id IN (SELECT id FROM drafts WHERE parent_id = ?1)
         ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;
    
    let link_iter = stmt.query_map([draft_id], |row| {
        Ok(DraftLink {
            id: row.get(0)?,
            draft_id: row.get(1)?,
            url_or_path: row.get(2)?,
            alias: row.get(3)?,
            created_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut links = Vec::new();
    for link in link_iter {
        links.push(link.map_err(|e| e.to_string())?);
    }
    Ok(links)
}

#[tauri::command]
pub fn add_draft_link(state: State<DbState>, draft_id: String, url_or_path: String, alias: Option<String>) -> Result<DraftLink, String> {
    let conn = state.conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO draft_links (id, draft_id, url_or_path, alias, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, draft_id, url_or_path, alias, now],
    ).map_err(|e| e.to_string())?;

    Ok(DraftLink {
        id,
        draft_id,
        url_or_path,
        alias,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_draft_link(state: State<DbState>, id: String, draft_id: String, url_or_path: String, alias: Option<String>) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "UPDATE draft_links SET draft_id = ?1, url_or_path = ?2, alias = ?3 WHERE id = ?4",
        params![draft_id, url_or_path, alias, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_draft_link(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute("DELETE FROM draft_links WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
