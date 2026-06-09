//! ContextFlow hotkey types and event bus.
//!
//! The crate intentionally does **not** own the platform-specific hotkey
//! registration code — that lives in `apps/desktop/src-tauri` because Tauri
//! supplies a maintained `tauri-plugin-global-shortcut` that handles the
//! Windows low-level keyboard hook lifecycle, refcounting, and DPI awareness.
//!
//! Instead, this crate defines the typed `HotkeyEvent` enum and the `HotkeyBus`
//! channel pair the rest of the engines consume. That way the dictation
//! orchestrator and any future test harness depend on this crate, not on Tauri.

pub mod binding;
pub mod bus;
pub mod event;

pub use binding::HotkeyBinding;
pub use bus::{HotkeyBus, HotkeyReceiver, HotkeySender};
pub use event::HotkeyEvent;
