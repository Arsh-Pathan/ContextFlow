//! Errors returned by the audio engine.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("no input audio device available")]
    NoDevice,

    #[error("failed to query default input config: {0}")]
    DefaultConfig(String),

    #[error("failed to build audio input stream: {0}")]
    BuildStream(String),

    #[error("failed to start audio input stream: {0}")]
    PlayStream(String),

    #[error("audio device callback reported an error: {0}")]
    StreamCallback(String),

    #[error("unsupported sample format from the input device: {0:?}")]
    UnsupportedSampleFormat(cpal::SampleFormat),

    #[error("resampler initialization failed: {0}")]
    Resampler(String),

    #[error("VAD initialization failed: {0}")]
    Vad(String),
}
