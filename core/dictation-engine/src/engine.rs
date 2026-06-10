//! The dictation state machine.
//!
//! One long-lived async task drives the lifecycle. It owns the optional
//! [`AudioEngine`] and [`SpeechSession`] for the *current* utterance, and
//! transitions between [`State::Idle`] and [`State::Listening`] in response
//! to [`HotkeyEvent`]s and speech-session events.
//!
//! The orchestrator does not register hotkeys or open windows itself — it
//! consumes a `HotkeyReceiver` from a [`HotkeyBus`] that the Tauri shell
//! fills, and it publishes [`DictationStatusEvent`]s through a
//! [`StatusEmitter`] callback the shell installs.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;
use tokio::time::{interval, Instant};
use tracing::{debug, error, info, warn};

use contextflow_audio_engine::AudioEngine;
use contextflow_hotkey::{HotkeyEvent, HotkeyReceiver};
use contextflow_ipc_contracts::{DictationStatus, DictationStatusEvent};
use contextflow_speech_engine::{SessionConfig, SpeechProvider, SpeechSession, TranscriptEvent};
use contextflow_text_injection::TextInjector;

use crate::error::DictationError;

/// How long we wait for a `Final` event after hotkey release before
/// giving up and emitting `NoTranscript`. The Windows recognizer usually
/// flushes its final within ~250 ms of `StopAsync`; 1500 ms is comfortably
/// past that and still well under "the user reaches for the mouse".
// 1.5s was too aggressive for Windows SR which can take up to 4-5s to finalize
// transcripts especially if it is using the cloud or there is background noise.
// 5s was too short for the WhisperCppProvider — CPU inference on the 488 MB
// ggml-base.en model can take 10-30+ seconds for a typical utterance.
const FINAL_TIMEOUT: Duration = Duration::from_secs(60);

/// Rate at which we re-emit `Listening { level }` events so the bubble
/// pulse looks lively without flooding Tauri's event channel.
const LEVEL_EMIT_INTERVAL: Duration = Duration::from_millis(50);

/// Callback the Tauri shell installs to publish a status event to the UI.
///
/// Boxed and `Send + Sync + 'static` so the orchestrator task can hold it
/// across awaits. Cloned per emission is cheap because it's an `Arc<…>` in
/// practice (the closure captures a `tauri::AppHandle` which is `Clone`).
pub type StatusEmitter = Arc<dyn Fn(DictationStatusEvent) + Send + Sync + 'static>;

/// Internal state — what phase of an utterance we're in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Listening,
}

/// Handle to a running orchestrator task. Drop it to stop the engine.
///
/// We keep the JoinHandle around so tests can `.await` shutdown; in
/// production code the engine runs for the life of the app and we never
/// drop it.
#[must_use = "dropping the DictationHandle aborts the orchestrator task"]
#[derive(Debug)]
pub struct DictationHandle {
    task: JoinHandle<()>,
}

impl DictationHandle {
    /// Abort the orchestrator task. Any in-flight session is dropped,
    /// which stops audio capture and ends the speech session cleanly.
    pub fn abort(self) {
        self.task.abort();
    }
}

/// Spin up the dictation orchestrator.
///
/// Spawns a tokio task that consumes `hotkey_rx`, runs the state machine,
/// and emits [`DictationStatusEvent`]s through `status_emit`. Returns a
/// [`DictationHandle`] the caller can drop to stop the task.
///
/// `provider` and `injector` are `Arc`'d so the orchestrator can share
/// them with internal tasks (e.g. the level-emitter) without taking
/// `&self` references across awaits.
#[derive(Debug)]
pub struct DictationEngine;

impl DictationEngine {
    /// Start the orchestrator. See module-level docs for the lifecycle.
    pub fn start(
        hotkey_rx: HotkeyReceiver,
        provider: Arc<dyn SpeechProvider>,
        injector: Arc<dyn TextInjector>,
        status_emit: StatusEmitter,
    ) -> DictationHandle {
        let task = tokio::spawn(run(hotkey_rx, provider, injector, status_emit));
        DictationHandle { task }
    }
}

async fn run(
    mut hotkey_rx: HotkeyReceiver,
    provider: Arc<dyn SpeechProvider>,
    injector: Arc<dyn TextInjector>,
    emit: StatusEmitter,
) {
    info!(
        provider = provider.id(),
        injector = injector.kind().as_str(),
        "dictation engine started"
    );
    emit(DictationStatusEvent::new(DictationStatus::Idle));

    let mut state = State::Idle;
    // The current utterance's resources. Both are `Some` only while
    // `state == Listening`.
    let mut current: Option<ActiveSession> = None;

    loop {
        let recv = hotkey_rx.recv().await;
        match recv {
            Ok(HotkeyEvent::Pressed) => {
                if state == State::Listening {
                    // User double-pressed without a release event reaching us.
                    // Tear down whatever we have and start fresh — better than
                    // wedging the engine in a half-state.
                    warn!("hotkey Pressed while already Listening; restarting session");
                    if let Some(session) = current.take() {
                        session.shutdown();
                    }
                }
                match start_session(&provider, emit.clone()).await {
                    Ok(active) => {
                        current = Some(active);
                        state = State::Listening;
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        error!(?e, "failed to start dictation session");
                        emit(DictationStatusEvent::error(msg));
                        // Stay Idle; next press will retry.
                        state = State::Idle;
                    }
                }
            }
            Ok(HotkeyEvent::Released) => {
                if state != State::Listening {
                    debug!("hotkey Released with no active session — ignored");
                    continue;
                }
                state = State::Idle;
                emit(DictationStatusEvent::new(DictationStatus::Processing));
                let Some(active) = current.take() else {
                    continue;
                };
                match active.finish(&injector).await {
                    Ok(()) => {
                        emit(DictationStatusEvent::new(DictationStatus::Idle));
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        error!(?e, "dictation session failed");
                        emit(DictationStatusEvent::error(msg));
                    }
                }
            }
            Err(RecvError::Lagged(skipped)) => {
                // We fell behind. Hotkey events are at finger speed; lagging
                // means the bus capacity is too small or a downstream task
                // got stuck. Log and continue from the next available event.
                warn!(skipped, "hotkey receiver lagged");
            }
            Err(RecvError::Closed) => {
                info!("hotkey bus closed; dictation engine shutting down");
                if let Some(session) = current.take() {
                    session.shutdown();
                }
                return;
            }
        }
    }
}

/// Everything a Listening utterance owns.
///
/// `AudioEngine` holds a `cpal::Stream` whose backing WASAPI handle is not
/// `Send` — so we **never** keep the engine itself on the orchestrator task.
/// Instead a dedicated audio task owns it for the session's lifetime, and
/// the orchestrator holds only a stop signal (`oneshot::Sender<()>`) plus
/// the task's `JoinHandle`. Dropping the sender (via `audio_stop.take()`)
/// or aborting the task tears the engine down.
struct ActiveSession {
    /// Drop sender to stop the audio task; `None` if audio failed to start.
    audio_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// Audio task handle for graceful shutdown.
    audio_task: Option<JoinHandle<()>>,
    /// The speech provider session.
    speech: SpeechSession,
    /// Handle to the task pumping RMS levels into the UI; aborted on
    /// shutdown so it doesn't keep emitting events past the session.
    level_task: JoinHandle<()>,
    /// When this session began, for latency telemetry in shutdown.
    started_at: Instant,
}

impl ActiveSession {
    /// On hotkey release: stop the audio task, wait for the final
    /// transcript with a timeout, inject it.
    async fn finish(mut self, injector: &Arc<dyn TextInjector>) -> Result<(), DictationError> {
        // Stop the level emitter first — Processing should look static.
        self.level_task.abort();
        // Signal the audio owner to drop the engine. cpal stream closes
        // immediately. The speech session keeps running until we drop it:
        // that's intentional, so the WinRT recognizer has a chance to
        // flush its pending final.
        if let Some(tx) = self.audio_stop.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.audio_task.take() {
            // Wait briefly for the audio task to release the device, but
            // don't block injection on it.
            let _ = tokio::time::timeout(Duration::from_millis(100), handle).await;
        }

        // Destructure SpeechSession to drop the audio sink immediately.
        // This signals "end of utterance" to the provider so it can flush
        // the final transcript without waiting for a natural pause.
        let contextflow_speech_engine::SpeechSession { audio_sink, mut events } = self.speech;
        drop(audio_sink);

        let final_text = wait_for_final(&mut events, FINAL_TIMEOUT).await?;

        // Drop the events stream before injection — provider's StopAsync
        // can take ~100 ms on Windows, and we'd rather pay it in parallel
        // with the SendInput call than serialise.
        drop(events);

        let trimmed = final_text.trim();
        if trimmed.is_empty() {
            debug!(
                latency_ms = self.started_at.elapsed().as_millis() as u64,
                "session ended with empty transcript"
            );
            return Ok(());
        }

        injector.inject(trimmed).await?;
        info!(
            chars = trimmed.chars().count(),
            latency_ms = self.started_at.elapsed().as_millis() as u64,
            "transcript injected"
        );
        Ok(())
    }

    /// Quick teardown without injecting — used when restarting a session
    /// because the user pressed twice without a release reaching us.
    fn shutdown(mut self) {
        self.level_task.abort();
        if let Some(tx) = self.audio_stop.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.audio_task.take() {
            handle.abort();
        }
        drop(self.speech);
    }
}

/// Build the resources for one utterance. Either everything starts or we
/// bail with `DictationError` and the orchestrator goes back to Idle.
async fn start_session(
    provider: &Arc<dyn SpeechProvider>,
    emit: StatusEmitter,
) -> Result<ActiveSession, DictationError> {
    let started_at = Instant::now();

    // Speech first: if it fails for a "speech privacy policy not accepted"
    // reason there's no point asking cpal to open the mic. The provider's
    // `start_session` returns a real `Result::Err` for init failures.
    let speech = provider.start_session(SessionConfig::default()).await?;

    // Audio capture for the bubble's level meter. cpal's `Stream` is `!Send`,
    // so we can't hold an `AudioEngine` across awaits on a multi-thread
    // runtime. Instead a dedicated task owns the engine: it starts it,
    let audio_sink = if provider.capabilities().feeds_own_audio {
        None
    } else {
        Some(speech.audio_sink.clone())
    };

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let (audio_task, level_task) = spawn_audio_owner(stop_rx, emit.clone(), audio_sink);

    debug!(
        startup_ms = started_at.elapsed().as_millis() as u64,
        "session started"
    );

    Ok(ActiveSession {
        audio_stop: Some(stop_tx),
        audio_task: Some(audio_task),
        speech,
        level_task,
        started_at,
    })
}

/// Spawn the audio-owning task plus the level-pump task.
///
/// Returns `(audio_task_handle, level_pump_handle)`. The audio task may
/// quickly conclude that cpal can't open the device; in that case it just
/// parks on the stop channel and the level pump emits zeros — silent
/// bubble, speech path unchanged.
fn spawn_audio_owner(
    stop_rx: tokio::sync::oneshot::Receiver<()>,
    emit: StatusEmitter,
    audio_sink: Option<contextflow_speech_engine::session::AudioSink>,
) -> (JoinHandle<()>, JoinHandle<()>) {
    // The level broadcast channel lives outside the audio task so we can
    // hand a Receiver to the level-pump task before the engine starts.
    // If audio fails, the broadcast just stays empty and the pump emits
    // zeros — exactly the silent-bubble behaviour we want.
    let (level_tx, level_rx) = tokio::sync::broadcast::channel::<f32>(8);

    // `AudioEngine` holds a `cpal::Stream` whose Windows backing handle is
    // `!Send`. We can't keep it across awaits on a multi-thread runtime,
    // so we run the audio owner on `spawn_blocking` and bridge the level
    // broadcast over an mpsc to the async world.
    let (audio_level_tx, mut audio_level_rx) = tokio::sync::mpsc::channel::<f32>(16);
    let (block_stop_tx, block_stop_rx) = std::sync::mpsc::sync_channel::<()>(1);

    // Hook the oneshot stop signal to the std mpsc the blocking thread
    // polls — small bridge task, exits when the oneshot fires.
    tokio::spawn(async move {
        let _ = stop_rx.await;
        let _ = block_stop_tx.send(());
    });

    let audio_task = tokio::task::spawn_blocking(move || {
        let engine = match AudioEngine::start() {
            Ok(engine) => engine,
            Err(e) => {
                warn!(?e, "audio engine failed to start; bubble will not pulse");
                // Park until told to stop so the JoinHandle resolves cleanly.
                let _ = block_stop_rx.recv();
                return;
            }
        };
        let mut levels = engine.subscribe_levels();
        let mut frames = audio_sink.as_ref().map(|_| engine.subscribe_frames());
        loop {
            if block_stop_rx.try_recv().is_ok() {
                break;
            }
            
            // Forward any available audio frames to the speech provider's sink
            if let Some(ref mut rx) = frames {
                while let Ok(frame) = rx.try_recv() {
                    if let Some(ref sink) = audio_sink {
                        let _ = sink.push_blocking(frame.samples);
                    }
                }
            }

            match levels.try_recv() {
                Ok(v) => {
                    // Best-effort forward to the async side; if the
                    // pump task is gone, the bubble is no longer
                    // listening for levels and we just drop the value.
                    let _ = audio_level_tx.try_send(v);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    // No new audio; nap briefly so we're not a hot spin.
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        // Dropping `engine` here releases the cpal stream.
        drop(engine);
    });

    // Forward the mpsc into the broadcast that level_pump consumes.
    let forward_tx = level_tx.clone();
    tokio::spawn(async move {
        while let Some(v) = audio_level_rx.recv().await {
            let _ = forward_tx.send(v);
        }
    });

    // The level pump throttles the RMS stream to ~20 Hz and emits
    // `Listening { level }` events to the UI.
    let level_pump = tokio::spawn(async move {
        let mut tick = interval(LEVEL_EMIT_INTERVAL);
        tick.tick().await; // skip the immediate first tick
        let mut latest: f32 = 0.0;
        let mut rx = level_rx;
        loop {
            tokio::select! {
                level = rx.recv() => {
                    match level {
                        Ok(v) => latest = v,
                        Err(RecvError::Lagged(_)) => {}
                        Err(RecvError::Closed) => {
                            // Engine task ended; emit one final zero-level
                            // event so the bubble doesn't get stuck at the
                            // last peak, then exit.
                            emit(DictationStatusEvent::listening(0.0));
                            return;
                        }
                    }
                }
                _ = tick.tick() => {
                    emit(DictationStatusEvent::listening(latest));
                }
            }
        }
    });

    (audio_task, level_pump)
}

/// Wait for a `Final` from the speech session with a timeout. Partials are
/// dropped — slice 1 doesn't show them in the UI. Errors from the session
/// surface immediately.
async fn wait_for_final(
    events: &mut futures::stream::BoxStream<'static, TranscriptEvent>,
    timeout: Duration,
) -> Result<String, DictationError> {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(DictationError::NoTranscript {
                timeout_ms: timeout.as_millis() as u64,
            });
        }
        let next = tokio::time::timeout(remaining, events.next()).await;
        match next {
            Err(_elapsed) => {
                return Err(DictationError::NoTranscript {
                    timeout_ms: timeout.as_millis() as u64,
                });
            }
            Ok(None) => {
                // Stream ended without a Final. Treat as empty transcript
                // (user said nothing); the orchestrator's caller will see
                // Ok("") and inject nothing.
                return Ok(String::new());
            }
            Ok(Some(TranscriptEvent::Final { text, .. })) => return Ok(text),
            Ok(Some(TranscriptEvent::Partial { .. })) => {}
            Ok(Some(TranscriptEvent::Empty)) => return Ok(String::new()),
            Ok(Some(TranscriptEvent::Error { message, .. })) => {
                return Err(DictationError::Speech(
                    contextflow_speech_engine::SpeechError::SessionEnded(message),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_transitions_are_distinct() {
        assert_ne!(State::Idle, State::Listening);
    }
}
