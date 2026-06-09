//! [`SpeechSession`], the per-utterance handle returned by a provider.

use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::SpeechError;

/// A single transcription session.
///
/// The session owns the resources needed to convert audio to text. Dropping
/// the session ends recognition and releases provider-side resources.
pub struct SpeechSession {
    /// Push audio frames here. Closing the sink signals end-of-utterance.
    pub audio_sink: AudioSink,
    /// Consume transcript events as they arrive.
    pub events: BoxStream<'static, TranscriptEvent>,
}

impl std::fmt::Debug for SpeechSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpeechSession")
            .field("audio_sink", &"<sender>")
            .field("events", &"<stream>")
            .finish()
    }
}

/// PCM audio frame: 16-bit signed mono samples at the rate declared in
/// [`SessionConfig::sample_rate_hz`](crate::provider::SessionConfig).
pub type AudioFrame = Vec<i16>;

/// Sink for PCM frames. Internally a tokio mpsc sender.
///
/// We wrap it so that providers can change the channel implementation later
/// (e.g. to `rtrb` for lock-free SPSC) without breaking call sites.
#[derive(Debug, Clone)]
pub struct AudioSink {
    inner: mpsc::Sender<AudioFrame>,
}

impl AudioSink {
    #[must_use]
    pub fn new(inner: mpsc::Sender<AudioFrame>) -> Self {
        Self { inner }
    }

    /// Push a frame. Returns `Err(SinkClosed)` if the consumer has gone away.
    pub async fn push(&self, frame: AudioFrame) -> Result<(), SpeechError> {
        self.inner
            .send(frame)
            .await
            .map_err(|_| SpeechError::SinkClosed)
    }

    /// Close the sink, signalling end-of-utterance. After this the session
    /// will emit a `Final` event and the event stream will end.
    pub fn close(self) {
        // Dropping the inner sender closes the channel.
        drop(self.inner);
    }
}

/// One observable event from a speech session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptEvent {
    /// In-progress transcript. May be revised by later partials or the final.
    Partial {
        text: String,
        /// 0.0 = very unstable, 1.0 = unlikely to change.
        stability: f32,
    },
    /// The provider's best guess for the full utterance.
    Final {
        text: String,
        #[serde(default)]
        alternatives: Vec<String>,
    },
    /// Recognition finished without any text (e.g. the user said nothing).
    Empty,
    /// The provider failed. `recoverable` tells the orchestrator whether to retry.
    Error { message: String, recoverable: bool },
}
