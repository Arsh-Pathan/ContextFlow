//! Voice Activity Detection — thin wrapper over `webrtc-vad`.
//!
//! `webrtc::Vad` holds a raw pointer to `fvad` and is therefore not `Send`.
//! We only ever use it from the dedicated worker task, so that's fine — the
//! wrapper just exists to localize the unsafe surface and centralize the
//! aggressive-mode/16 kHz/20 ms configuration so callers can't get it wrong.

use webrtc_vad::{SampleRate, Vad, VadMode};

use crate::error::AudioError;

pub(crate) struct VoiceDetector {
    inner: Vad,
}

impl std::fmt::Debug for VoiceDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceDetector").finish_non_exhaustive()
    }
}

impl VoiceDetector {
    /// Build a VAD configured for our pipeline: 16 kHz input, aggressive
    /// mode (mode 2). Aggressive is the right default for dictation —
    /// it favors fewer false positives, so background hum and HVAC don't
    /// trip a session. The user can swap modes in Slice 5 settings.
    ///
    /// `Result` for forward-compat: Slice 5 will accept a user-configured
    /// mode/rate combo that needs validation. Today it cannot fail.
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn new() -> Result<Self, AudioError> {
        let inner = Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Aggressive);
        Ok(Self { inner })
    }

    /// Classify exactly one 20 ms frame (320 samples at 16 kHz).
    ///
    /// Returns `false` on any internal VAD error — we treat detection
    /// failure as "not speech" rather than propagating, because a transient
    /// VAD glitch should not kill the capture pipeline.
    pub(crate) fn classify(&mut self, frame: &[i16]) -> bool {
        debug_assert_eq!(frame.len(), crate::frame::FRAME_SAMPLES);
        self.inner.is_voice_segment(frame).unwrap_or(false)
    }
}
