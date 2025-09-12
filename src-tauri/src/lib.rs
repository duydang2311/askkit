mod agent;
mod chat;
mod cipher;
mod common;
mod launcher;

use keyring::Entry;
use rand::{rngs::OsRng, TryRngCore};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{panic, sync::Arc};
use tauri::{
    generate_handler,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, RunEvent, Window, WindowEvent,
};
use uuid::Uuid;

use crate::{
    agent::{
        repo::{sqlite::SqliteAgentRepo, AgentRepo},
        AgentContext,
    },
    chat::repo::{sqlite::SqliteChatRepo, ChatRepo},
    cipher::{Cipher, KeyringAesGcmCipher},
    common::{
        error::AppError,
        http::HttpClientManager,
        unit_of_work::{SqliteUnitOfWorkFactory, UnitOfWorkFactory},
    },
};

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup)
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(handle_window_event)
        .invoke_handler(generate_handler![
            chat::cmds::create_chat,
            chat::cmds::send_chat_message,
            launcher::cmds::destroy_launcher_window,
            agent::cmds::get_agents,
            agent::cmds::get_current_agent,
            agent::cmds::update_current_agent,
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
    let app_local_data_dir = app
        .path()
        .app_local_data_dir()
        .expect("failed to get app local data dir");

    if !app_local_data_dir.exists() {
        std::fs::create_dir_all(&app_local_data_dir).expect("failed to create app local data dir");
    }

    let db_file_path = app_local_data_dir.join("askkit.db");
    let db_file_path_str = db_file_path
        .to_str()
        .expect("failed to convert path to str");

    let http_client_manager = Arc::new(HttpClientManager::new());

    tauri::async_runtime::block_on(async { Sqlite::create_database(db_file_path_str).await })
        .expect("failed to create sqlite database");

    let db_pool = Arc::new(tauri::async_runtime::block_on(async {
        SqlitePoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("pragma recursive_triggers = off")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&format!(
                "sqlite:{}",
                app_local_data_dir
                    .join("askkit.db")
                    .to_str()
                    .expect("failed to convert path to str"),
            ))
            .await
            .expect("failed to connect to sqlite")
    }));
    let cipher: Arc<dyn Cipher> = Arc::new(KeyringAesGcmCipher::new());
    let chat_repo: Arc<dyn ChatRepo> = Arc::new(SqliteChatRepo::new(db_pool.clone()));
    let agent_repo: Arc<dyn AgentRepo> =
        Arc::new(SqliteAgentRepo::new(db_pool.clone(), cipher.clone()));
    let unit_of_work_factory: Arc<dyn UnitOfWorkFactory> = Arc::new(SqliteUnitOfWorkFactory::new(
        db_pool.clone(),
        cipher.clone(),
    ));

    tauri::async_runtime::block_on(async { sqlx::migrate!("./migrations").run(&*db_pool).await })
        .expect("failed to run migrations");

    tauri::async_runtime::block_on(seed_data(db_pool.clone())).expect("failed to seed data");

    app.manage(AgentContext::new(
        http_client_manager.clone(),
        agent_repo.clone(),
        chat_repo.clone(),
    ));
    app.manage(db_pool);
    app.manage(cipher);
    app.manage(chat_repo);
    app.manage(agent_repo);
    app.manage(unit_of_work_factory);
    Ok(())
}

fn setup_keyring() -> Result<(), Box<dyn std::error::Error>> {
    let entry = Entry::new("askkit", "local")?;
    if let Err(e) = entry.get_secret() {
        if let keyring::Error::NoEntry = e {
            let mut key = [0u8; 32];
            OsRng.try_fill_bytes(&mut key)?;
            entry.set_secret(&key)?;
        } else {
            panic!("error while getting secret from keyring: {:?}", e);
        }
    }
    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    setup_dependencies(app)?;
    setup_tray_icon(app)?;
    setup_global_shortcut(app)?;
    setup_keyring()?;
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

async fn seed_data(db_pool: Arc<Pool<Sqlite>>) -> Result<(), AppError> {
    let mut tx = db_pool.begin().await.map_err(AppError::from)?;
    let count = sqlx::query_scalar::<_, i64>("select count(*) from agents")
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::from)?;
    if count == 0 {
        sqlx::query(
            r#"
            insert into agents (id, provider, model) values (?1, 'gemini', 'gemini-2.5-pro');
            insert into agents (id, provider, model) values (?2, 'gemini', 'gemini-2.5-flash');
            insert into agents (id, provider, model) values (?3, 'gemini', 'gemini-2.5-flash-lite');
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;
    }
    tx.commit().await.map_err(AppError::from)?;
    Ok(())
}
