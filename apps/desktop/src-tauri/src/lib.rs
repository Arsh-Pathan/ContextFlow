//! ContextFlow desktop shell entry point.
//!
//! The Tauri shell is intentionally thin — it owns window lifecycle, the
//! system tray, the global hotkey, and the IPC bridge to the UI, then
//! delegates everything else to the core engines under `core/`.
//!
//! Slice 1 progress:
//!   - commit 1: bubble window + tray icon          ✅
//!   - commit 2: Ctrl+Space hotkey + bubble state   ← this commit
//!   - commit 3: audio capture                      ⏳
//!   - commit 4: speech provider                    ⏳
//!   - commit 5: text injection                     ⏳
//!   - commit 6: dictation orchestrator             ⏳

mod hotkey_bridge;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
use tracing::info;

use contextflow_hotkey::HotkeyBus;

use crate::hotkey_bridge::HotkeyBridge;

/// Application entry point. Called from `main.rs` so the binary stays a thin
/// shim and the real bootstrap is testable from integration tests.
pub fn run() {
    contextflow_telemetry::install_dev_subscriber();
    info!("ContextFlow desktop starting");

    let hotkey_bus = HotkeyBus::new(16);

    tauri::Builder::default()
        .plugin(build_global_shortcut_plugin(hotkey_bus.sender()))
        .manage(hotkey_bus)
        .setup(|app| {
            build_tray(app)?;
            register_ptt_shortcut(app)?;
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

/// Build the global-shortcut plugin with our bridge as the handler.
///
/// The plugin handler receives `(AppHandle, &Shortcut, ShortcutEvent)` on
/// every press and release. We hand both off to [`HotkeyBridge::dispatch`],
/// which publishes into the in-process [`HotkeyBus`] and emits a typed Tauri
/// event so the React UI updates the bubble.
fn build_global_shortcut_plugin<R: tauri::Runtime>(
    hotkey_sender: contextflow_hotkey::HotkeySender,
) -> tauri::plugin::TauriPlugin<R> {
    let bridge = HotkeyBridge::new(hotkey_sender);
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, shortcut, event| {
            bridge.dispatch(app, shortcut, event);
        })
        .build()
}

/// Register `Ctrl+Space` as the push-to-talk hotkey.
///
/// Slice 1 hard-codes the binding. Slice 5 reads it from settings.
fn register_ptt_shortcut(app: &tauri::App) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let ctrl_space = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);

    // `register` returns the plugin's own error type, which doesn't `From`
    // into `tauri::Error`. Convert via `anyhow`, which does.
    app.global_shortcut()
        .register(ctrl_space)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("global hotkey: {e}")))?;

    info!(accelerator = "Ctrl+Space", "registered push-to-talk hotkey");
    Ok(())
}

