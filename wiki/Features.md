# Features

ContextFlow packs a robust set of features to make dictation seamless, fast, and beautiful.

## Core Engines
- **Capture**: Lock-free ring buffer audio capture using WASAPI loopback, with 16 kHz resampling via `rubato`, and voice activity detection via `webrtc-vad`.
- **Speech**: Pluggable provider architecture. Supports local `whisper.cpp` (CUDA-accelerated) and Windows SR, with planned support for faster-whisper, Deepgram, and OpenAI Realtime.
- **Injection**: Multi-layered injection strategy. Attempts UI Automation first, falls back to `SendInput`, and finally clipboard pasting if necessary.
- **AI Cleanup**: An intelligent AI pipeline capable of removing filler words, adding punctuation, and resolving on-the-fly spoken corrections. Supports local-first processing with options for OpenAI, Anthropic, Gemini, or Ollama.

## User Interface
- **Floating Bubble**: A beautiful, transparent, always-on-top dictation indicator with audio-reactive visualizers and state-driven animations.
- **Settings Panel**: A tray-accessible, WhisperFlow-style settings window with live cross-window syncing.
- **Theming**: Ships with 28 premium visual themes (e.g., Tokyo Night, Dracula, Cyberpunk) modifying the UI's colors and animations dynamically via a CSS-variable engine.

## Reliability & Privacy
- **Privacy-First**: Audio processing is fully on-device by default. Telemetry is opt-in only.
- **Global Hotkeys**: Reliable global hotkey (Ctrl+Space) with low-level keyboard hooks to detect releases even when other apps are focused.
- **Secure Storage**: API keys and secrets are securely stored in the Windows Credential Manager.
