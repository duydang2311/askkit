mod chat;
mod common;
mod launcher;

use crate::common::http::HttpClientManager;
use tauri::{
    generate_handler,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, RunEvent, WebviewWindow, Window, WindowEvent,
};
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg(desktop)]
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers};

#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_stronghold::Builder::new(|pass| todo!()).build())
        .setup(setup)
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(handle_window_event)
        .invoke_handler(generate_handler![
            chat::cmds::create_chat,
            chat::cmds::send_chat_message
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
        WindowEvent::Destroyed => {
            println!("window destroyed {:?}", window);
            if window.label() == "launcher" {
                let _ = launcher::shortcuts::unregister(window.app_handle().global_shortcut());
            }
        }
        WindowEvent::Focused(is_focused) => {
            if *is_focused {
                #[cfg(target_os = "windows")]
                let _ = apply_mica(window, None).inspect_err(|e| {
                    log::error!("failed to apply mica: {:?}", e);
                });
            }
            if window.label() == "launcher" {
                if *is_focused {
                    let _ = launcher::shortcuts::register(
                        window.app_handle().global_shortcut(),
                        |app_handle, shortcut, event| {
                            if event.state == ShortcutState::Released
                                && shortcut.matches(Modifiers::empty(), Code::Escape)
                            {
                                if let Some(window) = app_handle.get_webview_window("launcher") {
                                    let _ =
                                        destroy_launcher_webview_window(&window).inspect_err(|e| {
                                            log::error!(
                                                "error while destroying launcher window: {:?}",
                                                e
                                            );
                                        });
                                }
                            }
                        },
                    );
                } else {
                    let _ = destroy_launcher_window(window);
                    let _ = launcher::shortcuts::unregister(window.app_handle().global_shortcut());
                }
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

fn destroy_launcher_webview_window(window: &WebviewWindow) -> tauri::Result<()> {
    window.hide().inspect_err(|e| {
        log::error!("error while destroying launcher webview window: {:?}", e);
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
    app.manage(HttpClientManager::new());
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
