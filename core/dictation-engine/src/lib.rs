//! ContextFlow dictation orchestrator.
//!
//! Owns the per-session lifecycle: on hotkey-down, opens audio capture, starts
//! a speech session against the configured `SpeechProvider`, and on hotkey-up
//! hands the final transcript to the text injector for the focused window.
//!
//! See `ARCHITECTURE.md#dictation-session-lifecycle`.
//!
//! ## Status
//!
//! Slice 1 implementation lands once the audio, speech, and injection crates
//! have their real Slice 1 surfaces in place.
