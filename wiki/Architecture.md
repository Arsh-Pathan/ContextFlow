# Architecture

ContextFlow is structured as a **Rust workspace** running inside a **Tauri 2** desktop shell. The frontend is a minimal React overlay used strictly for UI elements (like the floating dictation bubble and settings window), while all heavy lifting (audio, speech, injection, hotkeys) happens in Rust.

## Topology

ContextFlow operates within a single OS process divided into three logical layers:

1. **UI (React in webview)**: Floating bubble, settings panel.
2. **Tauri Shell (Rust)**: Window lifecycle, global hotkey, IPC handlers.
3. **Core Engines (Rust crates)**: Dedicated components for audio, speech, AI, and text-injection.

## The SpeechProvider Abstraction
The heart of ContextFlow is the `SpeechProvider` trait (`core/speech-engine`).
Every speech engine implements the exact same interface, ensuring that the dictation orchestrator remains entirely decoupled from concrete providers. This allows you to hot-swap from local `whisper.cpp` to cloud-based Deepgram with zero changes to the capture or injection layers.

## Text-Injection Strategies
ContextFlow utilizes a fallback chain to guarantee text can be inserted anywhere:
1. **UI Automation**: Direct manipulation of `ValuePattern` text (browsers, Office).
2. **SendInput**: Synthesized `KEYEVENTF_UNICODE` keystrokes.
3. **Clipboard Paste**: Temporarily swapping clipboard contents to send `Ctrl+V` (often needed for terminals and Electron apps).

## Theming & Settings
- **Theming**: Handled entirely through semantic CSS tokens (`--cf-*`). Switching themes immediately updates the visualizer colors and UI style without affecting the React layout.
- **Settings**: Configuration is persisted to SQLite via Rust, with live multi-window updates driven by Tauri events.
