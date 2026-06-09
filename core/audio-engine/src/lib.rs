//! ContextFlow audio engine.
//!
//! Owns the microphone capture pipeline:
//!
//! ```text
//!   cpal callback (audio thread)
//!     │  f32 interleaved, device sample rate, N channels
//!     ▼  downmix → SPSC ring
//!   worker task (tokio blocking)
//!     │  drain ring → resample to 16 kHz → 20 ms frames → webrtc-vad
//!     ▼
//!   broadcast: AudioFrame { samples [i16 × 320], rms, voiced }
//!   broadcast: f32 level (RMS, 0..=1) for the UI meter
//! ```
//!
//! Downstream callers (the speech engine, the bubble's mic meter) consume
//! `AudioFrame`s and `f32` levels via `subscribe_frames()` and
//! `subscribe_levels()`. cpal types never escape this crate.
//!
//! Frame size is **20 ms at 16 kHz = 320 samples**. That's the size
//! `webrtc-vad` requires and a reasonable chunk for streaming providers.
//!
//! Dropping the [`AudioEngine`] stops capture and releases the cpal stream.

pub mod capture;
pub mod error;
pub mod frame;
pub mod resample;
pub mod vad;

pub use capture::AudioEngine;
pub use error::AudioError;
pub use frame::{AudioFrame, FRAME_SAMPLES, FRAME_SAMPLE_RATE_HZ};

/// Number of broadcast slots for the frame channel. ~20 frames = 400 ms of
/// buffering; if a consumer falls further behind it will lag (i.e. get the
/// `RecvError::Lagged` signal) rather than back-pressure the audio thread.
pub(crate) const FRAME_BROADCAST_CAPACITY: usize = 32;

/// Number of broadcast slots for the level meter. The UI samples this at
/// 30 Hz so we don't need much; 8 is plenty.
pub(crate) const LEVEL_BROADCAST_CAPACITY: usize = 8;
