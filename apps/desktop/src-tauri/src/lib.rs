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
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
use tracing::{error, info};

use contextflow_dictation_engine::{DictationEngine, DictationHandle, StatusEmitter};
use contextflow_hotkey::HotkeyBus;
use contextflow_ipc_contracts::{DictationStatusEvent, EVENT_DICTATION_STATUS};
use contextflow_speech_engine::providers::whisper_cpp::WhisperCppProvider;
use contextflow_text_injection::SendInputInjector;

use crate::hotkey_bridge::HotkeyBridge;

/// Application entry point. Called from `main.rs` so the binary stays a thin
/// shim and the real bootstrap is testable from integration tests.
pub fn run() {
    contextflow_telemetry::install_dev_subscriber();
    info!("ContextFlow desktop starting");

    let hotkey_bus = HotkeyBus::new(16);
    let hotkey_rx = hotkey_bus.subscribe();

    tauri::Builder::default()
        .plugin(build_global_shortcut_plugin(hotkey_bus.sender()))
        .manage(hotkey_bus)
        .setup(move |app| {
            build_tray(app)?;
            register_ptt_shortcut(app)?;
            
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

async fn ensure_model_exists<R: tauri::Runtime>(app: &AppHandle<R>) -> anyhow::Result<std::path::PathBuf> {
    let app_data = app.path().app_data_dir().map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data)?;
    let model_path = app_data.join("ggml-base.en.bin");
    
    if model_path.exists() {
        return Ok(model_path);
    }
    
    info!("Model not found at {:?}, downloading from HuggingFace...", model_path);
    let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
    
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
///   * `SendInputInjector` for typing the final transcript into the focused window.
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
            std::path::PathBuf::from("../../../ggml-base.en.bin")
        }
    };
    
    // Fall back to WindowsSpeechProvider if Whisper initialization fails (e.g. missing model)
    let provider: Arc<dyn contextflow_speech_engine::SpeechProvider> = match WhisperCppProvider::new(model_path) {
        Ok(p) => {
            info!("Successfully initialized WhisperCppProvider");
            Arc::new(p)
        }
        Err(e) => {
            error!("Failed to initialize WhisperCppProvider: {e}. Falling back to WindowsSpeechProvider.");
            Arc::new(contextflow_speech_engine::providers::windows_sr::WindowsSpeechProvider::new())
        }
    };
    
    let injector = Arc::new(SendInputInjector::new());

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
