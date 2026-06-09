//! ContextFlow settings store.
//!
//! Persists user configuration (audio device, speech provider, hotkeys,
//! cleanup level, dictionary, snippets, privacy flags) in SQLite under
//! `%LOCALAPPDATA%\ContextFlow\settings.db`. API keys go to the Windows
//! Credential Manager instead, never to SQLite.
//!
//! ## Status
//!
//! Real implementation begins in Slice 1 with a minimal config (hotkey,
//! provider id). Expanded in Slice 5.
