//! The PCM frame type the audio engine emits.
//!
//! 16 kHz mono `i16` samples, fixed at 20 ms per frame (320 samples). This
//! is what `webrtc-vad` requires and a reasonable chunk size for the speech
//! providers we ship.

use serde::{Deserialize, Serialize};

/// Sample rate every downstream consumer assumes.
pub const FRAME_SAMPLE_RATE_HZ: u32 = 16_000;

/// Samples per frame: 20 ms × 16 kHz = 320.
pub const FRAME_SAMPLES: usize = 320;

/// One short frame of mono PCM, plus the metadata producers care about.
///
/// `samples` is exactly [`FRAME_SAMPLES`] long. We use `Vec<i16>` rather than
/// `[i16; FRAME_SAMPLES]` so the broadcast channel can move it without
/// stack-copying 640 bytes per delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    /// 16-bit signed PCM at 16 kHz, mono. Always [`FRAME_SAMPLES`] long.
    pub samples: Vec<i16>,
    /// Root-mean-square amplitude in `0.0..=1.0`. Useful for mic meters
    /// without re-walking the samples downstream.
    pub rms: f32,
    /// VAD verdict: `true` means the frame contains speech.
    pub voiced: bool,
}

impl AudioFrame {
    /// Construct a frame and compute its RMS in one pass.
    ///
    /// Panics if `samples.len() != FRAME_SAMPLES`. We accept the panic
    /// because the only producer is internal to this crate and a wrong-sized
    /// frame is a bug, not a runtime failure.
    pub(crate) fn from_samples(samples: Vec<i16>, voiced: bool) -> Self {
        assert_eq!(
            samples.len(),
            FRAME_SAMPLES,
            "AudioFrame must contain exactly {FRAME_SAMPLES} samples"
        );
        let rms = rms_i16(&samples);
        Self {
            samples,
            rms,
            voiced,
        }
    }
}

/// Root-mean-square amplitude of an `i16` buffer, normalized to `0.0..=1.0`.
///
/// `i16::MAX = 32_767`. We divide by that, square-mean-root, and clamp.
/// Visible to the rest of the crate so the level meter can reuse it.
pub(crate) fn rms_i16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let scale = f64::from(i16::MAX);
    let sum_sq: f64 = samples
        .iter()
        .map(|&s| {
            let n = f64::from(s) / scale;
            n * n
        })
        .sum();
    let mean = sum_sq / samples.len() as f64;
    (mean.sqrt() as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_of_silence_is_zero() {
        let s = vec![0_i16; FRAME_SAMPLES];
        assert!(rms_i16(&s).abs() < 1e-6);
    }

    #[test]
    fn rms_of_full_scale_square_is_one() {
        let s: Vec<i16> = (0..FRAME_SAMPLES)
            .map(|i| if i.is_multiple_of(2) { i16::MAX } else { i16::MIN + 1 })
            .collect();
        let r = rms_i16(&s);
        // Within rounding distance of 1.0 — the alternating square wave is
        // very close to full amplitude.
        assert!(r > 0.99, "expected ~1.0, got {r}");
    }
}
