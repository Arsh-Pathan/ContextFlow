//! Bridges hotkey events from the `tauri-plugin-global-shortcut` callback
//! into both the in-process [`HotkeyBus`] (for the dictation orchestrator)
//! and a typed Tauri event (for the React bubble UI).
//!
//! Keeping this in its own file means `lib.rs` stays a flat composition of
//! setup steps and the actual fan-out logic is testable in isolation.

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutEvent, ShortcutState};
use tracing::{debug, error};

use contextflow_hotkey::{HotkeyEvent, HotkeySender};
use contextflow_ipc_contracts::{DictationStatus, DictationStatusEvent, EVENT_DICTATION_STATUS};

/// Owns the in-process publisher. Cloned into the plugin's handler closure.
#[derive(Debug, Clone)]
pub(crate) struct HotkeyBridge {
    sender: HotkeySender,
}

impl HotkeyBridge {
    pub(crate) fn new(sender: HotkeySender) -> Self {
        Self { sender }
    }

    /// Fan out a single platform hotkey callback.
    ///
    /// Translates `ShortcutState` → [`HotkeyEvent`], publishes on the bus,
    /// and emits a [`DictationStatusEvent`] to the bubble window so the UI
    /// reflects the press/release immediately.
    pub(crate) fn dispatch<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        shortcut: &Shortcut,
        event: ShortcutEvent,
    ) {
        let (hotkey_event, ui_status) = match event.state() {
            ShortcutState::Pressed => (HotkeyEvent::Pressed, DictationStatus::Listening),
            ShortcutState::Released => (HotkeyEvent::Released, DictationStatus::Idle),
        };

        debug!(
            shortcut = %shortcut.into_string(),
            ?hotkey_event,
            "hotkey transition"
        );

        // 1. Publish into the in-process bus so the dictation orchestrator
        //    (commit 6) can react. No subscribers yet in commit 2; that's fine.
        self.sender.send(hotkey_event);

        // 2. Emit a typed Tauri event so the bubble UI updates.
        let payload = DictationStatusEvent::new(ui_status);
        if let Err(err) = app.emit(EVENT_DICTATION_STATUS, &payload) {
            error!(?err, "failed to emit dictation status to UI");
        }
    }
}
