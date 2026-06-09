# Slice 1 Acceptance Test

**Goal:** prove the full ContextFlow loop works end-to-end on Windows.

Slice 1 ships:

- Tauri 2 desktop shell with system tray and a floating always-on-top bubble.
- Global `Ctrl+Space` push-to-talk hotkey.
- `cpal` microphone capture, resampled to 16 kHz mono.
- WebRTC VAD endpointing.
- `WindowsSpeechProvider` (via `Windows.Media.SpeechRecognition`).
- `SendInput` Unicode text injection.

Quality is rough; the loop is real. Polish and better speech land in Slice 2.

## Prerequisites

- Windows 10 (22H2+) or Windows 11.
- At least one English speech recognition language pack installed
  (`Settings → Time & language → Speech`).
- A working microphone (`Settings → System → Sound → Input`).
- ContextFlow built from source per the
  [Quickstart](../../README.md#quickstart-development).

## Test

1. **Launch.** Run `pnpm tauri dev` from the repo root.
   - **Expect:** within 2 seconds, the system tray shows the ContextFlow icon
     and a small floating bubble appears, anchored to the lower-right of the
     primary display, in the *idle* state (subtle pulse).

2. **Open Notepad.** Click into the document body so the caret is visible.

3. **Hold `Ctrl+Space`.**
   - **Expect:** the bubble enters the *listening* state within 50 ms.
   - **Expect:** the microphone meter on the bubble responds to your voice.

4. **Speak a short sentence:** *"Hello from ContextFlow."*

5. **Release `Ctrl+Space`.**
   - **Expect:** the bubble enters the *processing* state for under 1 second.
   - **Expect:** within ~1 s of release, the transcribed text appears in
     Notepad at the caret. Accuracy may be imperfect; "Hello from
     ContextFlow" recognized as "Hello from context flow" or similar is a
     pass.
   - **Expect:** the bubble returns to *idle*.

6. **Repeat** steps 3–5 three times. The loop must be reliable.

7. **Press `Ctrl+Space` while focused on another app** (e.g., Chrome address
   bar, VS Code editor). Slice 1 makes no guarantees about insertion in those
   apps — that's Slice 3 — but the bubble must still respond, and the app
   must not crash.

## Pass criteria

- All five "Expect" blocks pass on the first run after a fresh boot.
- The app does not crash, hang, or leak focus over 10 consecutive dictations.
- CPU usage in *idle* is under 5 % on a quad-core mid-range CPU.
- Memory under 400 MB resident.

## Known limitations (Slice 1)

- **Notepad only is guaranteed.** Other apps may or may not receive text
  cleanly. Slice 3 fixes this with UI Automation + per-app strategies.
- **No partial-result display.** The bubble shows *listening* / *processing*
  states but doesn't stream partial transcripts. Slice 2 adds streaming UI.
- **No AI cleanup.** Filler words ("uh", "um") will appear verbatim.
  Slice 4 adds the cleanup pipeline.
- **English (`en-US`) only.** Multi-language detection lands in Slice 5.
