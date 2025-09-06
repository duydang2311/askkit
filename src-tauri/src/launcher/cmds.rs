use tauri::{Runtime, WebviewWindow};

use crate::common::errors::AppError;

#[tauri::command]
pub fn destroy_launcher_window<R: Runtime>(webview_window: WebviewWindow<R>) -> Result<(), AppError> {
    webview_window
        .hide()
        .map_err(|e| AppError::Unknown(Some(Box::new(e))))
}
