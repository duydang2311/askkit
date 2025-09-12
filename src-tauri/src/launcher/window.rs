use tauri::{AppHandle, LogicalSize, Manager, WebviewUrl};

pub fn open(app_handle: &AppHandle) -> tauri::Result<()> {
    let window = match app_handle.get_webview_window("launcher") {
        Some(a) => Ok(a),
        None => {
            const WIDTH: f64 = 800.0;
            const HEIGHT: f64 = 64.0;
            let monitor_size = app_handle
                .primary_monitor()
                .ok()
                .flatten()
                .map(|a| a.work_area().size.to_logical(a.scale_factor()))
                .unwrap_or_else(|| LogicalSize::new(0.0, 0.0));
            tauri::WebviewWindowBuilder::new(
                app_handle,
                "launcher",
                WebviewUrl::App("/launcher".into()),
            )
            .title("Launcher")
            .inner_size(WIDTH, HEIGHT)
            .position(
                (monitor_size.width - WIDTH) / 2.0,
                (monitor_size.height - HEIGHT) * 0.25,
            )
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .build()
        }
    }?;

    if !window.is_visible()? {
        window.show()?;
    }
    if window.is_minimized()? {
        window.unminimize()?;
    }
    if !window.is_focused()? {
        window.set_focus()?;
    }
    Ok(())
}
