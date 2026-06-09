//! Typed IPC contracts shared between the Tauri Rust shell and the React UI.
//!
//! Types here derive `serde::Serialize`, `serde::Deserialize`, and
//! `specta::Type` so a single source of truth produces TypeScript bindings
//! consumed by the UI.
//!
//! ## Status
//!
//! Real types land alongside the Slice 1 Tauri shell. This crate is
//! intentionally empty until then.
