//! Resampling device-native rate (44.1 / 48 kHz typical) → 16 kHz mono.
//!
//! Wraps `rubato::FftFixedIn` so the rest of the engine speaks in flat
//! `Vec<f32>` slices and doesn't have to thread per-channel buffers around.

use rubato::{FftFixedIn, Resampler};

use crate::error::AudioError;

/// Fixed-input-chunk resampler from `src_rate` Hz mono → 16 kHz mono.
///
/// Built once at capture start, fed mono `f32` chunks of exactly
/// [`Self::chunk_size_in`] samples per call, returns 16 kHz `f32` chunks
/// of the size rubato chooses (we just collect them and re-chunk for VAD
/// downstream).
pub(crate) struct MonoResampler {
    inner: FftFixedIn<f32>,
    chunk_size_in: usize,
    // Scratch buffers reused across calls to avoid per-frame allocation.
    in_buf: Vec<Vec<f32>>,
    out_buf: Vec<Vec<f32>>,
}

impl MonoResampler {
    /// Build a resampler that takes `chunk_in` samples at `src_rate` and
    /// produces 16 kHz output.
    pub(crate) fn new(src_rate: u32, chunk_in: usize) -> Result<Self, AudioError> {
        let inner = FftFixedIn::<f32>::new(
            src_rate as usize,
            crate::frame::FRAME_SAMPLE_RATE_HZ as usize,
            chunk_in,
            /* sub_chunks */ 1,
            /* nbr_channels */ 1,
        )
        .map_err(|e| AudioError::Resampler(e.to_string()))?;

        let output_len = inner.output_frames_max();
        Ok(Self {
            inner,
            chunk_size_in: chunk_in,
            in_buf: vec![vec![0.0; chunk_in]; 1],
            out_buf: vec![vec![0.0; output_len]; 1],
        })
    }

    #[must_use]
    pub(crate) fn chunk_size_in(&self) -> usize {
        self.chunk_size_in
    }

    /// Resample exactly `chunk_size_in()` input samples into the output
    /// buffer, returning the actual number of output samples written.
    pub(crate) fn process(&mut self, mono_in: &[f32]) -> Result<&[f32], AudioError> {
        debug_assert_eq!(mono_in.len(), self.chunk_size_in);
        self.in_buf[0].clear();
        self.in_buf[0].extend_from_slice(mono_in);

        let (_in_frames, out_frames) = self
            .inner
            .process_into_buffer(&self.in_buf, &mut self.out_buf, None)
            .map_err(|e| AudioError::Resampler(e.to_string()))?;

        Ok(&self.out_buf[0][..out_frames])
    }
}
