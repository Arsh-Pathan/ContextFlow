//! ContextFlow desktop shell entry point.
//!
//! The Tauri shell is intentionally thin — it owns window lifecycle, the
//! system tray, IPC command surface, and the global hotkey registration,
//! then delegates everything else to the core engines under `core/`.
//!
//! Slice 1 wires the bubble window and the tray icon. Hotkey, audio, speech,
//! and injection land in their own commits.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tracing::info;

/// Application entry point. Called from `main.rs` so the binary stays a thin
/// shim and the real bootstrap is testable from integration tests.
pub fn run() {
    contextflow_telemetry::install_dev_subscriber();
    info!("ContextFlow desktop starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            build_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("ContextFlow: error while running tauri application");
}

/// Build the system tray icon and its context menu.
///
/// The menu lives entirely in the Rust shell — no IPC roundtrip — so it
/// responds even if the UI webview is busy.
fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit ContextFlow", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show bubble", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide bubble", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let _tray = TrayIconBuilder::with_id("contextflow-tray")
        .tooltip("ContextFlow")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                info!("tray: quit requested");
                app.exit(0);
            }
            "show" => {
                if let Some(w) = app.get_webview_window("bubble") {
                    let _ = w.show();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("bubble") {
                    let _ = w.hide();
                }
            }
            other => {
                tracing::warn!(menu_id = other, "unhandled tray menu event");
            }
        })
        .build(app)?;

    Ok(())
}
