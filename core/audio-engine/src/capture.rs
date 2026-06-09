//! Microphone capture orchestrator.
//!
//! Opens the system default input device via `cpal`, downmixes to mono on
//! the audio thread, ships PCM to a worker task that resamples to 16 kHz,
//! cuts the output into 20 ms frames, runs VAD on each, and broadcasts
//! [`AudioFrame`]s + RMS levels to subscribers.
//!
//! The split between audio thread and worker matters: cpal's callback
//! must return promptly or the device underruns. So the callback does the
//! minimum (downmix + send), and the worker does the slow work (FFT
//! resample, VAD).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use crate::error::AudioError;
use crate::frame::{rms_i16, AudioFrame, FRAME_SAMPLES, FRAME_SAMPLE_RATE_HZ};
use crate::resample::MonoResampler;
use crate::vad::VoiceDetector;
use crate::{FRAME_BROADCAST_CAPACITY, LEVEL_BROADCAST_CAPACITY};

/// Number of mono `f32` samples shipped from the audio thread to the worker
/// per buffer. We size this so the worker wakes ~30× per second at 48 kHz
/// (the most common device rate), which is plenty for sub-50 ms first-frame
/// latency without flooding the channel.
const AUDIO_CALLBACK_CHUNK: usize = 1_536;

/// Capacity (in chunks) of the audio-thread → worker mpsc.
const AUDIO_CHANNEL_CAPACITY: usize = 16;

/// Handle to the running audio engine. Drop it to stop capture.
///
/// The cpal stream lives inside the handle; dropping it stops the audio
/// thread immediately. The worker task observes the closed mpsc and exits.
pub struct AudioEngine {
    frames_tx: broadcast::Sender<AudioFrame>,
    levels_tx: broadcast::Sender<f32>,
    // Kept alive so dropping the engine drops the stream.
    _stream: Stream,
    // Signal to the worker task that we're shutting down (mpsc close also
    // works; this is a redundant safety net during graceful shutdown).
    shutdown: Arc<AtomicBool>,
}

impl std::fmt::Debug for AudioEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioEngine")
            .field("frames_rx_count", &self.frames_tx.receiver_count())
            .field("levels_rx_count", &self.levels_tx.receiver_count())
            .finish_non_exhaustive()
    }
}

impl AudioEngine {
    /// Open the system default input device and start capture.
    ///
    /// Returns an [`AudioEngine`] whose `subscribe_frames()` / `subscribe_levels()`
    /// produce the 16 kHz frames and RMS levels respectively. Drop the
    /// engine to stop capture.
    pub fn start() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or(AudioError::NoDevice)?;
        let device_name = device.name().unwrap_or_else(|_| "<unknown>".to_owned());

        let default_config = device
            .default_input_config()
            .map_err(|e| AudioError::DefaultConfig(e.to_string()))?;
        let sample_format = default_config.sample_format();
        let config: StreamConfig = default_config.into();
        let channels = config.channels as usize;
        let src_rate = config.sample_rate.0;

        info!(
            device = %device_name,
            sample_rate = src_rate,
            channels,
            ?sample_format,
            "opening audio input"
        );

        // Audio thread → worker channel. Carries mono f32 chunks.
        let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>(AUDIO_CHANNEL_CAPACITY);

        // Worker → subscribers broadcasts.
        let (frames_tx, _) = broadcast::channel::<AudioFrame>(FRAME_BROADCAST_CAPACITY);
        let (levels_tx, _) = broadcast::channel::<f32>(LEVEL_BROADCAST_CAPACITY);

        let shutdown = Arc::new(AtomicBool::new(false));

        // Spawn the worker on a blocking thread — rubato FFT + VAD are CPU
        // work that we'd rather not run on the async runtime's lightweight
        // task threads. We don't need tokio here; std::thread is enough.
        {
            let frames_tx = frames_tx.clone();
            let levels_tx = levels_tx.clone();
            let shutdown = shutdown.clone();
            std::thread::Builder::new()
                .name("contextflow-audio-worker".to_owned())
                .spawn(move || {
                    if let Err(err) =
                        worker_loop(audio_rx, src_rate, &frames_tx, &levels_tx, &shutdown)
                    {
                        error!(?err, "audio worker exited with error");
                    } else {
                        debug!("audio worker exited cleanly");
                    }
                })
                .map_err(|e| AudioError::BuildStream(format!("worker spawn: {e}")))?;
        }

        let stream = match sample_format {
            SampleFormat::F32 => build_stream_f32(&device, &config, channels, audio_tx)?,
            SampleFormat::I16 => build_stream_i16(&device, &config, channels, audio_tx)?,
            SampleFormat::U16 => build_stream_u16(&device, &config, channels, audio_tx)?,
            other => return Err(AudioError::UnsupportedSampleFormat(other)),
        };
        stream
            .play()
            .map_err(|e| AudioError::PlayStream(e.to_string()))?;

        Ok(Self {
            frames_tx,
            levels_tx,
            _stream: stream,
            shutdown,
        })
    }

    /// Subscribe to the 16 kHz mono frame stream. Late subscribers see only
    /// frames produced after they subscribed.
    #[must_use]
    pub fn subscribe_frames(&self) -> broadcast::Receiver<AudioFrame> {
        self.frames_tx.subscribe()
    }

    /// Subscribe to the RMS level stream (0.0..=1.0). Intended for UI meters
    /// — sample at ~30 Hz.
    #[must_use]
    pub fn subscribe_levels(&self) -> broadcast::Receiver<f32> {
        self.levels_tx.subscribe()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        // Stream is dropped next, which stops the audio callback. The
        // worker's mpsc closes when the last sender (held inside the stream
        // callback) goes away, and the worker thread exits.
    }
}

// ───────────────────────── audio thread builders ─────────────────────────

fn build_stream_f32(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    tx: mpsc::Sender<Vec<f32>>,
) -> Result<Stream, AudioError> {
    device
        .build_input_stream(
            config,
            move |data: &[f32], _| dispatch_f32(data, channels, &tx),
            |err| error!(?err, "cpal input stream error"),
            None,
        )
        .map_err(|e| AudioError::BuildStream(e.to_string()))
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    tx: mpsc::Sender<Vec<f32>>,
) -> Result<Stream, AudioError> {
    device
        .build_input_stream(
            config,
            move |data: &[i16], _| {
                let mut converted = Vec::with_capacity(data.len());
                for &s in data {
                    converted.push(f32::from(s) / f32::from(i16::MAX));
                }
                dispatch_f32(&converted, channels, &tx);
            },
            |err| error!(?err, "cpal input stream error"),
            None,
        )
        .map_err(|e| AudioError::BuildStream(e.to_string()))
}

fn build_stream_u16(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    tx: mpsc::Sender<Vec<f32>>,
) -> Result<Stream, AudioError> {
    device
        .build_input_stream(
            config,
            move |data: &[u16], _| {
                let mut converted = Vec::with_capacity(data.len());
                for &s in data {
                    // Map u16 [0..=65_535] → f32 [-1.0..=1.0].
                    let centered = i32::from(s) - 32_768;
                    converted.push(centered as f32 / 32_768.0);
                }
                dispatch_f32(&converted, channels, &tx);
            },
            |err| error!(?err, "cpal input stream error"),
            None,
        )
        .map_err(|e| AudioError::BuildStream(e.to_string()))
}

/// Audio-thread side of the pipeline. Downmix interleaved input to mono and
/// hand it to the worker. Must be cheap.
fn dispatch_f32(data: &[f32], channels: usize, tx: &mpsc::Sender<Vec<f32>>) {
    if channels == 0 || data.is_empty() {
        return;
    }
    let frames = data.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    if channels == 1 {
        mono.extend_from_slice(data);
    } else {
        let scale = 1.0 / channels as f32;
        for frame in data.chunks_exact(channels) {
            let sum: f32 = frame.iter().sum();
            mono.push(sum * scale);
        }
    }

    // `try_send` rather than `blocking_send`: if the worker is genuinely
    // behind we'd rather drop a buffer than stall the audio thread, which
    // would manifest as device underrun.
    if let Err(err) = tx.try_send(mono) {
        match err {
            mpsc::error::TrySendError::Full(_) => {
                warn!("audio worker behind; dropped a buffer");
            }
            mpsc::error::TrySendError::Closed(_) => {
                // Worker has exited; nothing to do — the engine is being
                // dropped.
            }
        }
    }
}

// ───────────────────────── worker thread ─────────────────────────

fn worker_loop(
    mut rx: mpsc::Receiver<Vec<f32>>,
    src_rate: u32,
    frames_tx: &broadcast::Sender<AudioFrame>,
    levels_tx: &broadcast::Sender<f32>,
    shutdown: &AtomicBool,
) -> Result<(), AudioError> {
    let mut resampler = MonoResampler::new(src_rate, AUDIO_CALLBACK_CHUNK)?;
    let mut vad = VoiceDetector::new()?;

    // Carryover for resampled-but-not-yet-frame-aligned samples.
    let mut tail_f32: Vec<f32> = Vec::with_capacity(FRAME_SAMPLES * 4);
    // Carryover for raw audio-thread chunks that don't fit the resampler's
    // fixed input size in one call.
    let mut pending_in: Vec<f32> = Vec::with_capacity(AUDIO_CALLBACK_CHUNK * 2);

    while let Some(chunk) = rx.blocking_recv() {
        if shutdown.load(Ordering::Acquire) {
            break;
        }
        pending_in.extend_from_slice(&chunk);

        // Feed the resampler in fixed-size chunks, accumulate output.
        while pending_in.len() >= resampler.chunk_size_in() {
            let take = resampler.chunk_size_in();
            let to_resample: Vec<f32> = pending_in.drain(..take).collect();
            let resampled = resampler.process(&to_resample)?;
            tail_f32.extend_from_slice(resampled);
        }

        // Cut into exact 320-sample i16 frames; run VAD; broadcast.
        while tail_f32.len() >= FRAME_SAMPLES {
            let slice: Vec<f32> = tail_f32.drain(..FRAME_SAMPLES).collect();
            let i16_frame: Vec<i16> = slice
                .iter()
                .map(|&f| {
                    let clamped = f.clamp(-1.0, 1.0);
                    (clamped * f32::from(i16::MAX)) as i16
                })
                .collect();
            let voiced = vad.classify(&i16_frame);

            // Push the RMS level first so the meter can move ahead of the
            // frame consumer if it wants to.
            let level = rms_i16(&i16_frame);
            let _ = levels_tx.send(level);

            let frame = AudioFrame::from_samples(i16_frame, voiced);
            let _ = frames_tx.send(frame);
        }
    }

    // ── Flush residual audio (utterance-end samples silently lost otherwise) ──
    // At most chunk_size_in() - 1 samples can be left in pending_in.
    if !pending_in.is_empty() {
        let chunk_in = resampler.chunk_size_in();
        pending_in.resize(chunk_in, 0.0);
        if let Ok(resampled) = resampler.process(&pending_in) {
            tail_f32.extend_from_slice(resampled);
        }
    }
    // At most FRAME_SAMPLES - 1 samples can be left in tail_f32.
    if !tail_f32.is_empty() {
        tail_f32.resize(FRAME_SAMPLES, 0.0);
        let slice: Vec<f32> = tail_f32.drain(..FRAME_SAMPLES).collect();
        let i16_frame: Vec<i16> = slice
            .iter()
            .map(|&f| {
                let clamped = f.clamp(-1.0, 1.0);
                (clamped * f32::from(i16::MAX)) as i16
            })
            .collect();
        let level = rms_i16(&i16_frame);
        let _ = levels_tx.send(level);
        let frame = AudioFrame::from_samples(i16_frame, false);
        let _ = frames_tx.send(frame);
    }

    // Confirm we're using the target sample rate as a sanity check (compile-time
    // unreachable if someone touches FRAME_SAMPLE_RATE_HZ without updating VAD).
    debug_assert_eq!(FRAME_SAMPLE_RATE_HZ, 16_000);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_downmixes_stereo_to_mono() {
        // Manual test of the downmix arithmetic without touching cpal.
        let stereo = vec![1.0_f32, -1.0, 0.5, -0.5];
        let (tx, mut rx) = mpsc::channel(4);
        dispatch_f32(&stereo, 2, &tx);
        let got = rx.try_recv().unwrap();
        assert_eq!(got, vec![0.0_f32, 0.0]);
    }

    #[test]
    fn dispatch_passes_mono_through() {
        let mono = vec![0.1_f32, -0.2, 0.3];
        let (tx, mut rx) = mpsc::channel(4);
        dispatch_f32(&mono, 1, &tx);
        let got = rx.try_recv().unwrap();
        assert_eq!(got, mono);
    }
}
