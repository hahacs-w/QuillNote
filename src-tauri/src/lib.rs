mod fs_cmds;
mod db_cmds;
mod config;

use std::sync::Mutex;
use std::str::FromStr;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, AppHandle,
};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState, GlobalShortcutExt};
use crate::config::ConfigState;

fn register_hotkey(app: &AppHandle, hotkey_str: &str) -> Result<(), String> {
    println!("Attempting to register hotkey: {}", hotkey_str);
    let shortcut = Shortcut::from_str(hotkey_str).map_err(|e| {
        let err = format!("Failed to parse hotkey '{}': {}", hotkey_str, e);
        eprintln!("{}", err);
        err
    })?;
    
    let _ = app.global_shortcut().unregister_all(); // Clear old ones
    app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            if let Some(window) = app.get_webview_window("main") {
                let is_visible = window.is_visible().unwrap_or(false);
                let is_focused = window.is_focused().unwrap_or(false);
                
                if is_visible && is_focused {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    }).map_err(|e| {
        let err = format!("Failed to register hotkey '{}': {}", hotkey_str, e);
        eprintln!("{}", err);
        err
    })?;
    
    println!("Hotkey '{}' registered successfully.", hotkey_str);
    Ok(())
}

#[tauri::command]
fn open_in_os(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Failed to open path: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_config = config::load_config();
    let conn = db_cmds::init_db(&app_config).expect("Failed to initialize database");
    
    let db_state = db_cmds::DbState {
        conn: Mutex::new(conn),
    };
    let config_state = config::ConfigState(Mutex::new(app_config));

    tauri::Builder::default()
        .manage(db_state)
        .manage(config_state)
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // Setup Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Register Initial Global Shortcut
            let hotkey = app.state::<ConfigState>().inner().0.lock().unwrap().hotkey.clone();
            let _ = register_hotkey(app.app_handle(), &hotkey);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fs_cmds::save_draft,
            fs_cmds::load_draft,
            fs_cmds::delete_draft_file,
            fs_cmds::get_draft_modified_time,
            fs_cmds::save_export_text,
            fs_cmds::save_export_binary,
            db_cmds::get_folders,
            db_cmds::create_folder,
            db_cmds::rename_folder,
            db_cmds::delete_folder,
            db_cmds::get_drafts,
            db_cmds::create_draft,
            db_cmds::get_sub_drafts,
            db_cmds::create_sub_draft,
            db_cmds::update_draft_meta,
            db_cmds::update_draft_timestamp,
            db_cmds::delete_draft_meta,
            db_cmds::get_tags,
            db_cmds::get_draft_tags,
            db_cmds::add_tag_to_draft,
            db_cmds::remove_tag_from_draft,
            db_cmds::get_draft_links,
            db_cmds::add_draft_link,
            db_cmds::update_draft_link,
            db_cmds::delete_draft_link,
            db_cmds::global_search,
            config::get_config,
            config::save_config,
            open_in_os
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Reopen { has_visible_windows, .. } = event {
                if !has_visible_windows {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}
