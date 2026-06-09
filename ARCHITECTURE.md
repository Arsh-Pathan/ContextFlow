# ContextFlow Architecture

This document is the source of truth for how ContextFlow is structured. Read
this before making cross-cutting changes. If you change the architecture,
update this document in the same PR.

## Goals (in priority order)

1. **Latency.** Visible-text latency from speech end to text-in-app under 300 ms
   on a mid-range CPU.
2. **Universality.** Text injection must work in every Windows text field —
   browsers, Office, VS Code, Slack, terminals, Win32 dialogs.
3. **Reliability.** A crash in any engine must not kill the app. Speech and
   injection failures are recoverable and observable.
4. **Replaceability.** Every external service (speech, AI cleanup, telemetry)
   sits behind a trait. Swapping providers is a configuration change.
5. **Privacy.** Audio never leaves the device unless a cloud provider is
   explicitly enabled. Secrets live in Windows Credential Manager.

## Topology

ContextFlow is a single OS process with three logical layers:

```text
┌──────────────────────────────────────────────────────────────────────┐
│                         UI (React in webview)                        │
│   floating bubble · settings panel · onboarding · diagnostics view   │
└──────────────────────────────────────────────────────────────────────┘
                                  ▲
                       Tauri IPC (typed contracts)
                                  ▼
┌──────────────────────────────────────────────────────────────────────┐
│                        Tauri shell (Rust)                            │
│   window/tray lifecycle · IPC handlers · global hotkey · watchdog    │
└──────────────────────────────────────────────────────────────────────┘
                                  ▲
                          internal event bus
                                  ▼
┌──────────────────────────────────────────────────────────────────────┐
│                      Core engines (Rust crates)                      │
│  audio · speech · text-injection · dictation · context · ai · hotkey │
│                          settings · telemetry                        │
└──────────────────────────────────────────────────────────────────────┘
```

There is no separate "speech daemon" or sidecar process in Slice 1. We may
introduce a watchdog process in Slice 6 for crash recovery, but the core
engines all run in-process on dedicated Tokio tasks.

## Workspace layout

```text
contextflow/
├── apps/
│   └── desktop/                  # Tauri 2 app
│       ├── src/                  # React UI (TypeScript)
│       └── src-tauri/            # Tauri Rust shell + IPC handlers
├── core/
│   ├── audio-engine/             # cpal capture, resampling, ring buffer, VAD
│   ├── speech-engine/            # SpeechProvider trait + providers
│   ├── text-injection/           # UIA / SendInput / clipboard strategies
│   ├── dictation-engine/         # Session orchestrator
│   ├── context-engine/           # Focused-window + app detection
│   ├── ai-engine/                # AI cleanup, voice commands
│   ├── hotkey/                   # Global hotkey + low-level keyboard hook
│   ├── settings/                 # SQLite-backed config + schema migration
│   └── telemetry/                # Logging, metrics, crash reporter
├── crates/
│   └── ipc-contracts/            # Shared types between Rust shell and TS UI
├── docs/
│   ├── acceptance/               # Per-slice acceptance tests
│   └── adr/                      # Architecture Decision Records
└── .github/workflows/            # CI: cargo check/test/clippy, tauri build
```

## Speech-provider abstraction

The most important design decision. Every speech engine is a `SpeechProvider`.

```rust
// core/speech-engine/src/provider.rs

#[async_trait::async_trait]
pub trait SpeechProvider: Send + Sync {
    /// Stable identifier (e.g. "windows", "whisper-cpp", "deepgram").
    fn id(&self) -> &'static str;

    /// Capabilities the provider supports.
    fn capabilities(&self) -> ProviderCapabilities;

    /// Begin a new dictation session. The provider returns a sink for PCM
    /// audio and a stream of transcription events. Both are dropped when the
    /// session ends.
    async fn start_session(
        &self,
        config: SessionConfig,
    ) -> Result<SpeechSession, SpeechError>;
}

pub struct SpeechSession {
    pub audio_sink: AudioSink,                // 16 kHz mono i16 frames
    pub events: BoxStream<'static, Event>,    // partial + final transcripts
}

pub enum Event {
    Partial { text: String, stability: f32 },
    Final   { text: String, alternatives: Vec<String> },
    Error   { source: SpeechError, recoverable: bool },
}
```

Concrete providers planned (Slice 1 ships only the first):

| Provider              | Slice | Notes                                                 |
|-----------------------|------:|-------------------------------------------------------|
| `WindowsSpeechProvider` | 1   | `Windows.Media.SpeechRecognition`. No deps, ships everywhere. |
| `WhisperCppProvider`    | 2   | `whisper-rs`, ggml `large-v3-turbo` default. Local-first.   |
| `FasterWhisperProvider` | 2   | Python sidecar via `tonic` gRPC. Higher accuracy.     |
| `DeepgramProvider`      | 4   | Streaming WebSocket. Cloud, opt-in.                   |
| `OpenAIRealtimeProvider`| 4   | Realtime API. Cloud, opt-in.                          |

The dictation orchestrator picks a provider from settings and never imports
concrete provider modules. The provider registry is built at startup from
features compiled in.

## Audio pipeline

1. **Capture** — `cpal` opens the default input device at its native sample
   rate (typically 44.1 / 48 kHz). We hold the stream open between sessions to
   avoid device-warmup latency.
2. **Resample** — `rubato` resamples to 16 kHz mono i16 — the lowest common
   denominator for speech engines.
3. **VAD** — `webrtc-vad` decides which frames carry speech. Silence beyond
   the configured timeout (default 700 ms) ends a session.
4. **Ring buffer** — A lock-free SPSC buffer (`rtrb`) holds ~5 s of audio so
   we can ship "pre-roll" frames to the speech provider once VAD fires.

The audio engine emits a single typed stream of `AudioFrame` values; speech
providers consume those frames and never see `cpal` directly.

## Dictation session lifecycle

```text
hotkey-down ─► dictation::Session::start()
                  │
                  ├── audio::begin_capture()
                  ├── speech::start_session(config)
                  ├── context::snapshot_focused_window()
                  │
                  ▼
              (loop: audio frames → speech → events → injection)
                  │
hotkey-up ────►   ├── audio::stop_capture()
                  ├── speech::end_session()
                  ├── ai::cleanup(final_text)         (Slice 4+)
                  └── injection::insert(text, target) (Slice 1 onward)
```

A session is a tokio task supervised by the dictation engine. Errors propagate
as `Event::Error { recoverable }`; the orchestrator decides whether to retry
or fail-fast and notify the UI.

## Text-injection strategies

Slice 1 ships `SendInput` Unicode injection only. Slice 3 adds the full chain:

1. **UI Automation (`uiautomation` crate)** — preferred. Sets `ValuePattern`
   text directly on the focused element. Works in browsers, Office, most Win32.
2. **SendInput synthesized keystrokes** — fallback. Emits Unicode `KEYEVENTF_UNICODE`
   events. Works everywhere but doesn't replace selections.
3. **Clipboard paste** — last resort. Stash the existing clipboard, set our
   text, send `Ctrl+V`, restore the clipboard. Used for terminals and apps
   where UIA and SendInput both fail (notably some Electron and JetBrains targets).

The injector picks a strategy from a per-app routing table keyed on the
focused window's process name and UIA control type.

## IPC contracts

The `crates/ipc-contracts` crate uses `serde` + `specta` to generate
TypeScript types from Rust definitions. Tauri commands are typed at both ends:

```rust
#[tauri::command]
#[specta::specta]
pub async fn dictation_status() -> DictationStatus { ... }
```

The UI consumes the generated `bindings.ts` and never hand-writes IPC types.

## Concurrency model

- **One Tokio runtime**, multi-threaded, built in `main`.
- Each engine exposes an `async` API and owns its tasks. Engines communicate
  through channels (`tokio::sync::mpsc`) — never through shared mutable state.
- The single piece of shared state is the settings store, which is read-mostly
  and guarded by `arc-swap` for lock-free reads.

## Error handling

- Library crates use `thiserror`-derived enums. Errors are typed and
  matchable. Source errors are preserved with `#[from]`.
- The binary crate (`apps/desktop/src-tauri`) uses `anyhow` only at the
  outermost boundary — for example when bubbling startup failures to a
  message-box error dialog.
- `unwrap()` and `expect()` are forbidden outside of tests and `main`'s
  startup sequence (where a panic is the correct response to a missing
  invariant like "no audio devices").

## Observability

- **Logging:** `tracing` with a JSON subscriber that writes to
  `%LOCALAPPDATA%\ContextFlow\logs\contextflow.log` (rotated daily, 7-day retention).
- **Metrics:** opt-in. We collect histogram timings for the four key spans:
  hotkey-to-capture, capture-to-first-partial, partial-to-final, final-to-injection.
- **Crash reporter:** `crashpad` integration lands in Slice 6.

## What is explicitly out of scope

- macOS and Linux support.
- A browser extension or web wrapper.
- Server-side state. ContextFlow is a single-user, single-device product.
- Multi-user accounts. Profiles are per-Windows-user.

## ADRs

Significant decisions live in [`docs/adr/`](./docs/adr/). When you make a
choice that future contributors will second-guess (e.g. "why webrtc-vad over
Silero?"), write an ADR.
