# ADR 0001 — `SpeechProvider` as the central abstraction

**Status:** accepted
**Date:** 2026-05-31

## Context

ContextFlow needs to support multiple speech recognition backends from
day one:

- `Windows.Media.SpeechRecognition` for the zero-dependency Slice 1 path.
- `whisper.cpp` (via `whisper-rs`) for the local-first default in Slice 2.
- `faster-whisper` as a higher-accuracy local option.
- `Deepgram` and `OpenAI Realtime` as cloud options.

We will also experiment with engines we haven't picked yet — Silero, Vosk,
on-device Gemini Live — without rewriting the dictation orchestrator.

If the dictation engine talks to a specific provider, swapping providers
requires touching every caller. That is the failure mode we are avoiding.

## Decision

The crate `core/speech-engine` defines a `SpeechProvider` trait. Every
backend implements it. The dictation orchestrator depends on
`Box<dyn SpeechProvider>`, never on a concrete provider.

The trait is small on purpose: `id`, `display_name`, `capabilities`,
`start_session(config) -> SpeechSession`. A session is a paired
`(AudioSink, BoxStream<TranscriptEvent>)`. Audio goes one way, events come
the other. The lifecycle of the underlying recognizer is tied to session
drop.

Concrete providers live under `core/speech-engine/src/providers/<name>.rs`
and are gated by Cargo features (`provider-windows`, `provider-whisper-cpp`,
…). A release build only links the providers that have been explicitly
enabled.

## Consequences

**Good:**

- Slice 2's whisper.cpp introduction is a new file under `providers/`. The
  orchestrator does not change.
- Cloud providers can be omitted from the binary entirely for users who want
  a guaranteed-offline build.
- Unit tests can use an in-memory mock provider without any audio plumbing.

**Bad:**

- Trait abstractions force every provider into the same lifecycle. Streaming
  cloud providers (Deepgram) and one-shot offline providers (whisper.cpp on a
  buffered utterance) must both fit the `AudioSink` + `BoxStream` shape. We
  may need to add capability flags as we discover differences.
- Hosting WinRT inside the trait is non-trivial because the WinRT recognizer
  prefers to own its own audio path. Slice 1's Windows provider takes a
  pragmatic shortcut (lets WinRT open the mic directly while the rest of the
  pipeline runs unused for that provider). Slice 2 switches to the audio
  engine's frames via `MediaStreamSource`.

## Alternatives considered

- **Pick one provider and hard-code it.** Faster initial code; impossible to
  swap later without a rewrite. Rejected — the brief explicitly requires
  provider abstraction.
- **Process-per-provider sidecar.** Each provider runs as a separate process
  with a gRPC API. Cleaner isolation, much more operational complexity.
  Considered for `faster-whisper` (which needs Python anyway) but not as the
  general pattern.
