//! ContextFlow audio engine.
//!
//! Owns the microphone capture pipeline: `cpal` → resampler → VAD → ring
//! buffer → typed `AudioFrame` stream. Downstream callers (the speech engine,
//! waveform UI) never see `cpal` types.
//!
//! See `ARCHITECTURE.md#audio-pipeline` for the design.
//!
//! ## Status
//!
//! Slice 1 implementation lands once the local MSVC + Windows SDK toolchain
//! is installed. The crate compiles as an empty library until then; nothing
//! depends on its surface yet.
