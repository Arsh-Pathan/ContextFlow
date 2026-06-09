//! Typed IPC contracts shared between the Tauri Rust shell and the React UI.
//!
//! Types here derive `serde::Serialize` and `serde::Deserialize`. A future
//! commit (Slice 1 commit 6) hooks `specta-typescript` to generate
//! `apps/desktop/src/bindings.ts` automatically; for now the TypeScript side
//! re-declares matching types in `apps/desktop/src/ipc.ts` and we keep them
//! in sync by hand. The shape is small enough for that to be safe.

pub mod events;

pub use events::{DictationStatus, DictationStatusEvent, EVENT_DICTATION_STATUS};
