//! The [`SpeechProvider`] trait. See module docs in `lib.rs`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::SpeechError;
use crate::session::SpeechSession;

/// Capabilities a provider declares at registration time.
///
/// The dictation orchestrator uses these to filter providers by user
/// settings (e.g. "only local providers") and to decide whether features
/// like streaming partials are usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// Whether the provider streams partial transcripts before the final result.
    pub streaming_partials: bool,
    /// Whether audio leaves the device (true for cloud providers).
    pub network_required: bool,
    /// Whether the provider supports a "whisper mode" — low-amplitude speech.
    pub whisper_mode: bool,
    /// Whether the provider can auto-detect the spoken language.
    pub auto_language: bool,
    /// Whether the provider uses the GPU (advisory; not a guarantee).
    pub gpu_capable: bool,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            streaming_partials: false,
            network_required: false,
            whisper_mode: false,
            auto_language: false,
            gpu_capable: false,
        }
    }
}

/// Per-session configuration handed to the provider at `start_session` time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// BCP-47 language tag (e.g. `"en-US"`). `None` means auto-detect, when supported.
    pub language: Option<String>,
    /// PCM sample rate of the frames pushed to the audio sink.
    pub sample_rate_hz: u32,
    /// Whether to emit partial results.
    pub emit_partials: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            language: Some("en-US".to_owned()),
            sample_rate_hz: 16_000,
            emit_partials: true,
        }
    }
}

/// A speech recognition backend.
///
/// Implementations must be `Send + Sync` because providers are shared across
/// tasks. A provider is a long-lived value; per-utterance state lives in the
/// [`SpeechSession`] it returns from [`SpeechProvider::start_session`].
#[async_trait]
pub trait SpeechProvider: Send + Sync {
    /// Stable identifier for settings and logs (e.g. `"windows"`, `"whisper-cpp"`).
    fn id(&self) -> &'static str;

    /// Human-readable name for the settings UI.
    fn display_name(&self) -> &'static str;

    /// Capabilities, used for UI filtering and orchestrator decisions.
    fn capabilities(&self) -> ProviderCapabilities;

    /// Begin a dictation session. The caller must push 16-bit mono PCM frames
    /// at `config.sample_rate_hz` to the returned [`AudioSink`] and consume
    /// transcript events from the session's event stream.
    async fn start_session(&self, config: SessionConfig) -> Result<SpeechSession, SpeechError>;
}
