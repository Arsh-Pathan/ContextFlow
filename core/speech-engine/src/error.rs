//! Errors returned by speech providers.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpeechError {
    #[error("the requested provider `{0}` is not available in this build")]
    ProviderUnavailable(&'static str),

    #[error("provider failed to initialize: {0}")]
    InitFailed(String),

    #[error("microphone or audio device error: {0}")]
    Audio(String),

    #[error("recognition session ended unexpectedly: {0}")]
    SessionEnded(String),

    #[error("the audio sink is closed; cannot accept more frames")]
    SinkClosed,

    #[error("network error while talking to a cloud provider: {0}")]
    Network(String),

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("provider returned an unrecoverable error: {0}")]
    Fatal(String),
}

impl SpeechError {
    /// Whether the dictation orchestrator should attempt to restart the session.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::SessionEnded(_) | Self::Network(_) | Self::SinkClosed
        )
    }
}
