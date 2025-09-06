mod chat;
mod common;
mod launcher;

use std::sync::Arc;

use crate::common::{agent::AgentContext, http::HttpClientManager};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite};
use tauri::{
    generate_handler,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, RunEvent, Window, WindowEvent,
};

#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_stronghold::Builder::new(|pass| todo!()).build())
        .setup(setup)
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(handle_window_event)
        .invoke_handler(generate_handler![
            chat::cmds::create_chat,
            chat::cmds::send_chat_message,
            launcher::cmds::destroy_launcher_window
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(handle_run_event);
}

fn handle_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            if window.label() == "launcher" {
                let _ = destroy_launcher_window(window);
            } else {
                let _ = window.destroy().inspect_err(|e| {
                    log::error!("error while destroying window: {:?}", e);
                });
            }
        }
        WindowEvent::Focused(true) => {
            #[cfg(target_os = "windows")]
            let _ = apply_mica(window, None).inspect_err(|e| {
                log::error!("failed to apply mica: {:?}", e);
            });
        }
        _ => {}
    }
}

fn destroy_launcher_window(window: &Window) -> tauri::Result<()> {
    window.hide().inspect_err(|e| {
        log::error!("error while destroying launcher window: {:?}", e);
    })
}

fn setup_tray_icon(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|icon, event| match event {
            TrayIconEvent::DoubleClick { .. } => {
                let _ = launcher::window::open(icon.app_handle()).inspect_err(|e| {
                    log::error!("error while opening launcher window: {:?}", e);
                });
            }
            _ => {}
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn setup_global_shortcut(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["super+/"])?
                .with_handler(move |app_handle, shortcut, event| {
                    if event.state == ShortcutState::Released {
                        if shortcut.matches(Modifiers::SUPER, Code::Slash) {
                            let _ = launcher::window::open(app_handle).inspect_err(|e| {
                                log::error!("error while opening launcher window: {:?}", e);
                            });
                        }
                    }
                })
                .build(),
        )?;
    }
    Ok(())
}

fn setup_dependencies(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
    }

    let db_file_path = app_data_dir.join("askkit.db");
    let db_file_path_str = db_file_path
        .to_str()
        .expect("failed to convert path to str");

    let http_client_manager = Arc::new(HttpClientManager::new());

    tauri::async_runtime::block_on(async { Sqlite::create_database(db_file_path_str).await })
        .expect("failed to create sqlite database");

    let db_pool = Arc::new(tauri::async_runtime::block_on(async {
        SqlitePoolOptions::new()
            .connect(&format!(
                "sqlite:{}",
                app_data_dir
                    .join("askkit.db")
                    .to_str()
                    .expect("failed to convert path to str"),
            ))
            .await
            .expect("failed to connect to sqlite")
    }));

    tauri::async_runtime::block_on(async { sqlx::migrate!("./migrations").run(&*db_pool).await })
        .expect("failed to run migrations");

    app.manage(AgentContext::new(
        http_client_manager.clone(),
        db_pool.clone(),
    ));
    app.manage(db_pool.clone());
    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    setup_dependencies(app)?;
    setup_tray_icon(app)?;
    setup_global_shortcut(app)?;
    Ok(())
}

fn handle_run_event(_app_handle: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { api, code, .. } = event {
        match code {
            Some(code) => {
                log::info!("exit code: {:?}", code);
            }
            None => {
                api.prevent_exit();
            }
        }
    }
}
