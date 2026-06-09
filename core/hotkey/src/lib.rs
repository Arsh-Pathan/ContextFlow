//! ContextFlow global hotkey manager.
//!
//! Registers a single system-wide push-to-talk hotkey (default `Ctrl+Space`)
//! via the `global-hotkey` crate, and supplements it with a low-level
//! keyboard hook (`WH_KEYBOARD_LL`) so we can distinguish hold-to-talk from
//! tap-to-toggle without owning input focus.
//!
//! ## Status
//!
//! Slice 1 implementation lands once the toolchain is ready.
