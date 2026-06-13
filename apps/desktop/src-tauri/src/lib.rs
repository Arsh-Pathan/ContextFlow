//! ContextFlow desktop shell entry point.
//!
//! The Tauri shell is intentionally thin — it owns window lifecycle, the
//! system tray, the global hotkey, and the IPC bridge to the UI, then
//! delegates everything else to the core engines under `core/`.
//!
//! Slice 1 progress:
//!   - commit 1: bubble window + tray icon          ✅
//!   - commit 2: Ctrl+Space hotkey + bubble state   ✅
//!   - commit 3: audio capture                      ✅
//!   - commit 4: speech provider                    ✅
//!   - commit 5: text injection                     ✅
//!   - commit 6: dictation orchestrator             ← this commit

mod hotkey_bridge;

use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use tracing::{error, info};

use contextflow_dictation_engine::{DictationEngine, DictationHandle, StatusEmitter};
use contextflow_hotkey::HotkeyBus;
use contextflow_ipc_contracts::{DictationStatus, DictationStatusEvent, EVENT_DICTATION_STATUS};
use contextflow_speech_engine::providers::whisper_cpp::WhisperCppProvider;
use contextflow_text_injection::ClipboardInjector;

use crate::hotkey_bridge::HotkeyBridge;

use tauri_plugin_autostart::ManagerExt;

// Application entry point...
pub fn run() {
    contextflow_telemetry::install_dev_subscriber();
    info!("ContextFlow desktop starting");

    let hotkey_bus = HotkeyBus::new(16);
    let hotkey_rx = hotkey_bus.subscribe();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .plugin(build_global_shortcut_plugin(hotkey_bus.sender()))
        .invoke_handler(tauri::generate_handler![open_settings, close_settings])
        .manage(hotkey_bus)
        .setup(move |app| {
            // Enable autostart
            let autostart_manager = app.autolaunch();
            if let Err(e) = autostart_manager.enable() {
                tracing::warn!("Failed to enable autostart: {}", e);
            }

            build_tray(app)?;

            let handle_for_shortcuts = app.handle().clone();
            register_ptt_shortcut(&handle_for_shortcuts);

            // Spawn a background task to detect sleep/wake cycles and re-register the hotkey.
            // Windows frequently drops global shortcuts on wake from sleep or if the thread stalls.
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                let mut last_tick = std::time::SystemTime::now();
                loop {
                    interval.tick().await;
                    let now = std::time::SystemTime::now();
                    if let Ok(duration) = now.duration_since(last_tick) {
                        if duration > std::time::Duration::from_secs(15) {
                            tracing::info!("Detected system wake or time jump. Re-registering hotkey...");
                            register_ptt_shortcut(&handle_for_shortcuts);
                        }
                    }
                    last_tick = now;
                }
            });

            // DictationEngine::start calls tokio::spawn, which requires a Tokio context.
            // Tauri setup runs synchronously on the main thread, so we enter the runtime context
            // by using block_on.
            let handle = tauri::async_runtime::block_on(async {
                start_dictation_engine(app.handle().clone(), hotkey_rx).await
            });

            // Stash the handle in Tauri state so a future `quit` path can
            // call `.abort()`. For slice 1 the engine simply runs until the
            // process exits.
            app.manage(DictationOrchestrator(handle));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("ContextFlow: error while running tauri application");
}

/// New-type wrapper so the `DictationHandle` can live in Tauri's state map.
struct DictationOrchestrator(#[allow(dead_code)] DictationHandle);

async fn ensure_model_exists<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> anyhow::Result<std::path::PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {e}"))?;
    std::fs::create_dir_all(&app_data)?;
    let model_path = app_data.join("ggml-large-v3-turbo.bin");

    if model_path.exists() {
        return Ok(model_path);
    }

    // Check if model is bundled as a Tauri resource
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled = resource_dir.join("ggml-large-v3-turbo.bin");
        if bundled.exists() {
            info!(
                "Copying bundled model from {:?} to {:?}",
                bundled, model_path
            );
            std::fs::copy(&bundled, &model_path)?;
            return Ok(model_path);
        }
    }

    info!(
        "Model not found at {:?}, downloading from HuggingFace...",
        model_path
    );
    let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin";

    let mut response = reqwest::get(url).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download model: {}", response.status());
    }

    let mut file = tokio::fs::File::create(&model_path).await?;
    while let Some(chunk) = response.chunk().await? {
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }

    info!("Model downloaded successfully to {:?}", model_path);
    Ok(model_path)
}

/// Spin up the dictation orchestrator with the slice-2 backends:
///   * `WhisperCppProvider` for transcription (whisper-rs).
///   * `ClipboardInjector` for typing the final transcript into the focused window.
///   * A status-emit closure that forwards every `DictationStatusEvent`
///     to the bubble UI via the same Tauri event channel the hotkey bridge uses.
async fn start_dictation_engine<R: tauri::Runtime>(
    app: AppHandle<R>,
    hotkey_rx: contextflow_hotkey::HotkeyReceiver,
) -> DictationHandle {
    // Ensure the model exists in the app data directory, downloading it if necessary
    let model_path = match ensure_model_exists(&app).await {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to ensure model exists: {e}. Falling back to default path.");
            std::path::PathBuf::from("../../../ggml-large-v3-turbo.bin")
        }
    };

    // Fall back to WindowsSpeechProvider if Whisper initialization fails (e.g. missing model)
    info!(model = ?model_path, "initializing dictation provider");
    let (provider, provider_warning): (
        Arc<dyn contextflow_speech_engine::SpeechProvider>,
        Option<String>,
    ) = match WhisperCppProvider::new(model_path) {
        Ok(p) => {
            info!("Successfully initialized WhisperCppProvider");
            (Arc::new(p), None)
        }
        Err(e) => {
            let warning = format!("Whisper model failed to load: {e}. Falling back to Windows Speech Recognition — accuracy may be degraded.");
            error!("{warning}");
            (
                Arc::new(
                    contextflow_speech_engine::providers::windows_sr::WindowsSpeechProvider::new(),
                ),
                Some(warning),
            )
        }
    };
    info!(provider_id = provider.id(), "dictation provider selected");

    // Emit an initial Idle event so the bubble shows which provider is active.
    let mut init_event =
        DictationStatusEvent::new(DictationStatus::Idle).with_provider(provider.id());
    if let Some(ref w) = provider_warning {
        init_event = init_event.with_warning(w);
    }
    let _ = app.emit(EVENT_DICTATION_STATUS, &init_event);

    let injector = Arc::new(ClipboardInjector::new());

    let emit: StatusEmitter = Arc::new(move |event: DictationStatusEvent| {
        if let Err(err) = app.emit(EVENT_DICTATION_STATUS, &event) {
            error!(?err, "failed to emit dictation status from orchestrator");
        }
    });

    info!("starting dictation orchestrator");
    DictationEngine::start(hotkey_rx, provider, injector, emit)
}

/// Build the system tray icon and its context menu.
///
/// The menu lives entirely in the Rust shell — no IPC roundtrip — so it
/// responds even if the UI webview is busy.
fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit ContextFlow", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show bubble", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide bubble", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &show, &hide, &quit])?;

    let _tray = TrayIconBuilder::with_id("contextflow-tray")
        .tooltip("ContextFlow")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                show_settings_window(app);
            }
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

/// Show and focus the settings window, creating nothing (it is declared in
/// `tauri.conf.json` as a hidden window). Centralised so both the tray menu
/// and the IPC command share one path.
fn show_settings_window<R: tauri::Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.unminimize();
        if let Err(e) = w.set_focus() {
            tracing::warn!("failed to focus settings window: {e}");
        }
    } else {
        tracing::warn!("settings window not found");
    }
}

/// IPC command so the UI (e.g. an in-app button) can open settings too.
#[tauri::command]
fn open_settings(app: AppHandle) {
    show_settings_window(&app);
}

/// IPC command to hide the settings window (its custom titlebar close button).
#[tauri::command]
fn close_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.hide();
    }
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
fn register_ptt_shortcut(app: &tauri::AppHandle) {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let ctrl_space = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);

    // Unregister first in case it's in a stuck state
    let _ = app.global_shortcut().unregister(ctrl_space);

    // `register` returns the plugin's own error type.
    // Instead of failing the entire app startup, log a warning if it fails 
    // (e.g., if another instance is already running).
    if let Err(e) = app.global_shortcut().register(ctrl_space) {
        tracing::warn!("Failed to register push-to-talk hotkey (Ctrl+Space): {e}. Is ContextFlow already running?");
    } else {
        info!(accelerator = "Ctrl+Space", "registered push-to-talk hotkey");
    }
}
