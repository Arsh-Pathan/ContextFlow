# Changelog

All notable changes to ContextFlow are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
once it reaches 1.0. Pre-1.0 versions may break compatibility between minor
releases.

## [Unreleased]

### Added

- **Settings window** — a WhisperFlow-style preferences surface opened from the
  tray ("Settings…"), with a custom titlebar, sidebar navigation, and five
  panels: General, Appearance, AI Provider, Features, and About. Themed live and
  synced across windows.
- **Theme system** — 28 hand-tuned themes (ContextFlow Dark, Tokyo Night,
  Dracula, Nord, Solarized, Cyberpunk, White Flames, Midnight Neon, Oceanic,
  Black & White, Minimal Dark, and more) spanning six families, plus light
  variants. Themes change colour and motion only — never layout — via `--cf-*`
  CSS custom properties and eight CSS-only motion personalities. The Appearance
  panel previews each as a live miniature bubble. `contextflow-dark` is the
  default and is identical to the previous look.
- **AI Provider configuration** — choose the clarification/cleanup backend:
  Built-in (on-device, default), OpenAI, Anthropic, Gemini, or Ollama, with
  per-provider model, API key, and base-URL fields, plus a cleanup-level
  control. AI clarification is opt-in (off by default).
- **Feature flags** — an opt-in registry (AI clarification, voice commands,
  personal dictionary, live transcript preview, ambient background, sound cues,
  snippets, per-app profiles, dictation history). Every flag defaults to off, so
  the out-of-the-box experience is unchanged.
- Reduce-motion preference (independent of the OS setting) honoured across the
  bubble and settings window.
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
