# ContextFlow

> Your thoughts, in flow.

Windows-native AI voice dictation. Press a hotkey, speak naturally, and have text
appear in any application — faster than typing, smarter than the built-in
dictation, and available everywhere.

ContextFlow is a Windows 10/11 desktop app that combines low-latency speech
recognition, an AI cleanup pipeline, and a robust text-injection layer to let
you dictate into any text field on the system.

> **Status:** pre-alpha. Slice 1 (end-to-end thin vertical: hotkey → audio → VAD →
> transcription → injection into Notepad) is under active development. The
> public API surface is unstable and will change without notice.

---

## Why

Typing is a bottleneck. Voice is faster, but Windows' built-in dictation only
works in some apps, has noticeable latency, and doesn't clean up filler words,
auto-punctuate, or understand spoken corrections.

ContextFlow is built to:

- Work in **every** Windows text field (browsers, Office, VS Code, Slack, terminals, Win32 dialogs).
- Stream transcription with **<300 ms** visible latency.
- Run **fully offline** by default. Optional cloud providers are pluggable.
- Polish raw speech into clean writing via an AI cleanup layer that understands
  spoken corrections ("…meet at 2, actually 3" → "meet at 3").
- Stay invisible until you need it, then stay out of the way.

## Architecture

ContextFlow is a Cargo workspace with a Tauri 2 desktop shell and a Rust core
split into focused engines. The frontend is React + Tailwind + shadcn.

```text
apps/
  desktop/                 Tauri 2 shell + React UI (settings, floating bubble)
core/
  audio-engine/            cpal capture, resampling, VAD, ring buffer
  speech-engine/           SpeechProvider trait + concrete providers
  text-injection/          UIA / SendInput / clipboard strategies + per-app routing
  dictation-engine/        Session orchestrator: hotkey → capture → speech → injection
  context-engine/          Focused-window + input-field detection, per-app profiles
  ai-engine/               AI cleanup + voice-command provider abstraction
  hotkey/                  Global hotkey registration + low-level keyboard hook
  settings/                Persisted config (SQLite + serde)
  telemetry/               Opt-in metrics, structured logging, crash reporting
crates/
  ipc-contracts/           Typed Tauri command/event contracts shared with the UI
```

The single most important architectural decision is the **`SpeechProvider`
trait** in `core/speech-engine`. Every speech engine — Windows built-in,
whisper.cpp, faster-whisper, Deepgram, OpenAI Realtime — implements the same
trait. The dictation orchestrator never sees a concrete provider. Engines can
be swapped at runtime without touching capture, VAD, or injection code.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full design.

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for the slice plan. We ship in vertical slices,
each of which is independently runnable and acceptance-tested on Windows.

| Slice | Goal | Status |
|------:|------|--------|
| 1 | End-to-end thin vertical: hotkey → Notepad | 🚧 In progress |
| 2 | Local speech pipeline (whisper.cpp, streaming, models) | ⏳ Planned |
| 3 | Robust text injection (UIA, per-app strategies) | ⏳ Planned |
| 4 | AI cleanup + voice commands | ⏳ Planned |
| 5 | Context engine, snippets, personal dictionary, settings UI | ⏳ Planned |
| 6 | Reliability, watchdog, installer, auto-update, telemetry | ⏳ Planned |

## Quickstart (development)

### Prerequisites

- **Windows 11** or Windows 10 (22H2+)
- **Rust** stable (`rustup toolchain install stable`)
- **Node.js 20+** and **pnpm 9+** (`npm install -g pnpm`)
- **Visual Studio 2022 Build Tools** with the C++ workload and the Windows 11 SDK
- **CMake** (only required once Slice 2's whisper.cpp provider lands)

### First-time setup

```powershell
# Clone
git clone https://github.com/<your-org>/contextflow.git
cd contextflow

# Install JS deps
pnpm install

# Verify the workspace compiles
cargo check --workspace

# Run the app in dev mode (hot-reload UI + Rust)
pnpm tauri dev
```

### Slice 1 acceptance test

1. Run `pnpm tauri dev`. The floating bubble appears, system tray icon shows.
2. Open **Notepad**. Click into the document so the caret is in the text area.
3. Hold **Ctrl + Space**. The bubble enters the *listening* state.
4. Speak a short sentence: *"Hello from ContextFlow."*
5. Release Ctrl + Space. Within ~1 s, the transcribed text appears in Notepad.

If any of those steps fails, that's a Slice 1 bug — please file an issue or
check [`docs/troubleshooting.md`](./docs/troubleshooting.md).

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). The short version:

- Commits follow [Conventional Commits](https://www.conventionalcommits.org/).
- Branches: `main` is stable, `dev` is integration, feature work happens on
  `feature/<slice>-<short-name>` branches.
- Pre-commit runs `cargo fmt`, `cargo clippy -D warnings`, `cargo check`,
  `pnpm lint`, and `pnpm typecheck`. CI re-runs these plus the full test suite.

## Security and privacy

- All speech audio stays on-device by default. No audio leaves your machine
  unless you explicitly enable a cloud speech provider.
- API keys are stored via the **Windows Credential Manager**, never in plain
  text on disk.
- Telemetry is **opt-in** and limited to anonymized performance metrics.
- See [`docs/security.md`](./docs/security.md) for the full threat model.

## License

Apache License 2.0. See [LICENSE](./LICENSE).
