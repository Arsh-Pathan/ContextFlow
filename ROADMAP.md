# ContextFlow Roadmap

ContextFlow ships in **vertical slices**. Each slice is end-to-end runnable,
independently acceptance-tested, and demonstrably valuable. A slice is not
"done" until its acceptance test passes on a clean Windows 11 install.

## Slice 1 — Foundation + end-to-end thin vertical 🚧

**Goal:** prove the full loop works on Windows. Press a hotkey, speak, see text
in Notepad. Quality is rough; the loop is real.

**Scope:**

- Tauri 2 desktop shell with system tray and floating always-on-top bubble.
- Global `Ctrl+Space` hotkey (push-to-talk).
- Microphone capture via `cpal`, resampled to 16 kHz mono.
- WebRTC VAD for endpointing.
- `SpeechProvider` trait + a single concrete implementation: `WindowsSpeechProvider`
  (using `Windows.Media.SpeechRecognition`).
- Text injection via `SendInput` Unicode keystrokes.
- Cargo workspace + GitHub Actions CI (`cargo check`, `test`, `clippy`, frontend build, Tauri build validation).

**Acceptance test** — [`docs/acceptance/slice-1.md`](./docs/acceptance/slice-1.md):

1. Run `pnpm tauri dev`. Floating bubble appears, tray icon visible.
2. Focus Notepad.
3. Hold `Ctrl+Space`. Bubble shows *listening*.
4. Speak: "Hello from ContextFlow."
5. Release. Text appears in Notepad within ~1 s.

**Out of scope (deferred to later slices):**

- Streaming partial-result display.
- AI cleanup of filler words.
- Apps other than Notepad.
- whisper.cpp.
- UIA injection.
- Polish (animations, dark mode, onboarding).

## Slice 2 — Better local speech pipeline ⏳

**Goal:** ship a local-first, low-latency, high-accuracy speech pipeline that
runs entirely offline.

**Scope:**

- `WhisperCppProvider` (via `whisper-rs`), default model `large-v3-turbo`.
- `FasterWhisperProvider` via a Python sidecar (gRPC over Unix-domain pipe).
- Model manager: first-run download with progress UI, integrity checks, cache.
- Streaming transcription with partial results displayed in the bubble.
- VAD improvements: configurable thresholds, "whisper mode" (lower energy floor + noise suppression).
- GPU acceleration via CUDA / DirectML when available; clean CPU fallback.
- Benchmark harness comparing providers on a fixed corpus: latency, accuracy (WER), memory, CPU/GPU.
- Latency instrumentation: span timings exported as Prometheus-style histograms.

**Acceptance test:** [`docs/acceptance/slice-2.md`](./docs/acceptance/slice-2.md)
WER ≤ 8% on `librispeech-test-clean-small`, p95 first-partial latency ≤ 400 ms on
a Ryzen 5 5600 / i5-12400-class CPU.

## Slice 3 — Robust text injection ⏳

**Goal:** insertion that works in every real app, with diagnostics when it doesn't.

**Scope:**

- UI Automation primary path (`uiautomation` crate, `ValuePattern` and `TextPattern`).
- `SendInput` Unicode fallback with selection-replace support.
- Clipboard paste last-resort with clipboard preservation.
- Per-app strategy routing table keyed on process name and UIA control type.
- Verified targets:
  - Chrome (regular fields + Google Docs)
  - Microsoft Word
  - VS Code + Cursor
  - Slack desktop
  - Discord desktop
  - Outlook
  - Windows Terminal
  - JetBrains IDEs
  - Notepad / WordPad
- Insertion diagnostics: per-attempt log entry describing which strategy was
  used, latency, and failure reason.

**Acceptance test:** [`docs/acceptance/slice-3.md`](./docs/acceptance/slice-3.md)
Dictate the same sentence into each verified target. All must show the text correctly.

## Slice 4 — AI cleanup + voice commands ⏳

**Goal:** raw transcripts become polished writing, and voice commands transform
selected text.

**Scope:**

- `AiProvider` trait with implementations:
  - `OpenAiProvider` (Chat Completions + Realtime)
  - `AnthropicProvider`
  - `GeminiProvider`
  - `OllamaProvider` (local)
- Streaming post-processing pipeline:
  - filler-word removal
  - punctuation and capitalization
  - spoken-correction handling ("…at 2, actually 3" → "at 3")
  - grammar fixes
- Voice commands: "make professional", "shorter", "bullet points", "convert to email", "fix grammar", "explain simpler".
- API keys stored via Windows Credential Manager (`keyring` crate).
- Cleanup-level setting: off / light / standard / aggressive.

## Slice 5 — Context, snippets, dictionary, settings UI ⏳

**Goal:** the app feels personal and configurable.

**Scope:**

- `ContextEngine`: focused-window detection, per-app profiles (code / email / chat / general).
- Personal dictionary with learn-from-corrections.
- Snippets / voice macros with variable substitution.
- Multi-language auto-detection (Whisper handles the language ID).
- Full settings panel (audio devices, providers, hotkeys, cleanup, dictionary, snippets, privacy, diagnostics).

## Slice 6 — Reliability, installer, telemetry ⏳

**Goal:** product is ready for non-developer users.

**Scope:**

- Watchdog process that restarts the main app on crash.
- Crashpad-based crash reporting (opt-in upload).
- Auto-update via Tauri updater + signed binaries.
- Inno Setup installer with code signing.
- Recovery from: sleep/wake, mic disconnect, focus changes, frozen apps, Windows updates.
- Opt-in telemetry: anonymized latency histograms and error counts.
- Onboarding flow.

## Non-goals

- Cross-platform support (macOS, Linux, web, browser extension).
- Multi-tenant / cloud sync.
- Hosted backend services.
