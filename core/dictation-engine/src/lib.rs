//! ContextFlow dictation orchestrator.
//!
//! Owns the per-session lifecycle that ties the engines together:
//!
//! ```text
//!   HotkeyBus (Ctrl+Space)        SpeechProvider session
//!         │                              ▲
//!         │  Pressed → enter Listening   │  events (Partial, Final, …)
//!         ▼                              │
//!   DictationEngine ─────────────────────┘
//!         │                              │
//!         │  on Final → inject text      ▼
//!         │  on Released → enter Idle  AudioEngine ── RMS levels
//!         ▼
//!   StatusEmitter (DictationStatusEvent) → bubble UI
//! ```
//!
//! Slice 1 ships only the hold-to-talk path:
//!
//! * **Pressed**: spin up `AudioEngine` (for the bubble's RMS meter) and a
//!   `SpeechProvider` session. Emit `Listening`. Start a background task
//!   that re-emits `Listening { level }` events at ~20 Hz so the bubble
//!   pulses with the live mic.
//! * **Released**: emit `Processing`. Drop the audio engine. Wait up to
//!   1.5 s for a `Final` transcript from the speech session, then inject
//!   it via `TextInjector`. Emit `Idle`.
//! * Anywhere in there an `Error` from any subsystem becomes a single
//!   `Error { message }` event and we return to `Idle`.
//!
//! The `Windows.Media.SpeechRecognition` provider declares
//! `feeds_own_audio = true`; for that backend we deliberately do **not**
//! push our PCM into its `audio_sink`. We still spin our `AudioEngine`
//! up so the bubble's RMS meter works. When a future provider sets
//! `feeds_own_audio = false`, the orchestrator will additionally pipe
//! frames from `AudioEngine::subscribe_frames` into `session.audio_sink`.

#![doc(html_root_url = "https://docs.rs/contextflow-dictation-engine")]

pub mod engine;
pub mod error;

pub use engine::{DictationEngine, DictationHandle, StatusEmitter};
pub use error::DictationError;
