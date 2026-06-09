//! ContextFlow text injection.
//!
//! Inserts text into the focused window on Windows. Slice 1 ships
//! `SendInput` Unicode keystroke injection only. Slice 3 adds UI Automation
//! as the primary path and a clipboard-paste last-resort fallback, with
//! per-app strategy routing.
//!
//! See `ARCHITECTURE.md#text-injection-strategies`.
//!
//! ## Status
//!
//! Slice 1 implementation lands once the local MSVC + Windows SDK toolchain
//! is installed.
