//! ContextFlow speech engine.
//!
//! The single most important type here is the [`SpeechProvider`] trait.
//! Every speech recognition backend — Windows built-in, whisper.cpp,
//! faster-whisper, Deepgram, OpenAI Realtime — implements this trait.
//! Downstream code in `dictation-engine` never imports a concrete provider;
//! it picks one from settings and consumes the trait.
//!
//! See `ARCHITECTURE.md` for the rationale.

pub mod error;
pub mod provider;
pub mod session;

// Concrete providers live behind crate features so an unfinished or
// unavailable backend cannot accidentally be linked into a release build.
#[cfg(feature = "provider-windows")]
pub mod providers {
    pub mod windows_sr;
}

pub use error::SpeechError;
pub use provider::{ProviderCapabilities, SessionConfig, SpeechProvider};
pub use session::{AudioFrame, AudioSink, SpeechSession, TranscriptEvent};
