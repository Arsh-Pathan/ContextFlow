# Changelog

All notable changes to ContextFlow are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
once it reaches 1.0. Pre-1.0 versions may break compatibility between minor
releases.

## [Unreleased]

### Added

- Initial repository scaffolding: Cargo workspace layout, Tauri 2 shell, React
  + Tailwind UI scaffold.
- Apache-2.0 license, contributor guide, architecture and roadmap docs.
- GitHub Actions CI matrix: `cargo check`, `cargo test`, `cargo clippy`,
  frontend build, Tauri build validation.
- `SpeechProvider` trait in `core/speech-engine` with `WindowsSpeechProvider`
  as the first implementation.
- Audio engine: `cpal` capture, rubato resampling to 16 kHz mono, WebRTC VAD.
- Global hotkey `Ctrl+Space` (push-to-talk) via `global-hotkey` crate.
- Floating always-on-top dictation bubble with idle / listening / processing
  states.
- Text injection via `SendInput` Unicode keystrokes (Slice 1 fallback path).

[Unreleased]: https://github.com/your-org/contextflow/compare/HEAD...HEAD
