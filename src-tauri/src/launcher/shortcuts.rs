use std::sync::LazyLock;
use tauri::{AppHandle, Runtime};

#[cfg(desktop)]
use tauri_plugin_global_shortcut::{Code, GlobalShortcut, Shortcut, ShortcutEvent};

static ESCAPE_LAUNCHER_SHORTCUT: LazyLock<Shortcut> =
    LazyLock::new(|| Shortcut::new(None, Code::Escape));

#[cfg(desktop)]
pub fn register<R, F>(
    global_shortcut: &GlobalShortcut<R>,
    handler: F,
) -> Result<(), tauri_plugin_global_shortcut::Error>
where
    R: Runtime,
    F: Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static,
{
    global_shortcut
        .on_shortcut(*ESCAPE_LAUNCHER_SHORTCUT, handler)
        .inspect_err(|e| {
            log::error!("error while registering escape launcher shortcut: {:?}", e);
        })
}

#[cfg(desktop)]
pub fn unregister<R: Runtime>(
    global_shortcut: &GlobalShortcut<R>,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    global_shortcut
        .unregister(*ESCAPE_LAUNCHER_SHORTCUT)
        .inspect_err(|e| {
            log::error!(
                "error while unregistering escape launcher shortcut: {:?}",
                e
            );
        })
}
