//! Errors the dictation orchestrator surfaces.
//!
//! Most user-visible failures originate in a child engine (audio, speech,
//! injection); the orchestrator wraps them with context about which session
//! phase was active. The `Display` text of each variant is what the bubble
//! shows in its tooltip and what we send on `DictationStatusEvent::error`.

use thiserror::Error;

use contextflow_audio_engine::AudioError;
use contextflow_speech_engine::SpeechError;
use contextflow_text_injection::InjectionError;

#[derive(Debug, Error)]
pub enum DictationError {
    #[error("audio engine: {0}")]
    Audio(#[from] AudioError),

    #[error("speech engine: {0}")]
    Speech(#[from] SpeechError),

    #[error("text injection: {0}")]
    Injection(#[from] InjectionError),

    #[error("no transcript arrived within {timeout_ms} ms after hotkey release")]
    NoTranscript { timeout_ms: u64 },
}

impl DictationError {
    /// Whether the orchestrator should automatically return to Idle and
    /// accept the next hotkey press. For slice 1 every variant is
    /// recoverable — we never "lock" the engine on a failure.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        true
    }
}
