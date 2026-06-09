//! `Windows.Media.SpeechRecognition` provider — the slice 1 backend.
//!
//! This wraps Microsoft's built-in speech recognizer. It ships with every
//! supported Windows install (a one-time English-US language pack may need
//! to be enabled in Settings → Time & Language), runs entirely offline,
//! and requires no model download. It is therefore the right choice for
//! "fastest path to a working end-to-end demo" — the engineering rule
//! the rest of slice 1 is built around.
//!
//! ## Architectural caveat — `feeds_own_audio`
//!
//! The Windows API insists on opening its own microphone. We do **not** feed
//! PCM into it. This is the one provider in the workspace that ignores the
//! [`AudioSink`](crate::AudioSink). The orchestrator still runs our
//! [`contextflow_audio_engine`](https://docs.rs/contextflow-audio-engine)
//! pipeline so the bubble's RMS meter and VAD-driven endpointing keep
//! working — we just don't push PCM into the provider.
//!
//! Every other provider (whisper.cpp, faster-whisper, Deepgram, OpenAI
//! Realtime) consumes our PCM directly and reports `feeds_own_audio: false`.
//! See `docs/adr/0001-speech-provider-trait.md`.
//!
//! ## Threading
//!
//! `SpeechRecognizer` is a WinRT type; its methods must be called on a COM
//! MTA thread, and its events fire on internal worker threads. We isolate
//! that here:
//!
//! * `start_session` spawns a dedicated `std::thread` ("contextflow-winsr").
//! * That thread calls `RoInitialize(MTA)`, constructs the recognizer,
//!   compiles constraints, and starts `ContinuousRecognitionSession`.
//! * Event handler closures push [`TranscriptEvent`]s into an mpsc.
//! * The session's `events` stream is a `ReceiverStream` over that mpsc.
//! * Dropping the session sends a stop signal; the worker calls `StopAsync`,
//!   uninitializes COM, and exits.

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};
use windows::core::HSTRING;
use windows::Foundation::TypedEventHandler;
use windows::Media::SpeechRecognition::{
    SpeechContinuousRecognitionCompletedEventArgs,
    SpeechContinuousRecognitionResultGeneratedEventArgs, SpeechContinuousRecognitionSession,
    SpeechRecognitionHypothesisGeneratedEventArgs, SpeechRecognitionResultStatus, SpeechRecognizer,
    SpeechRecognizerStateChangedEventArgs,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use crate::error::SpeechError;
use crate::provider::{ProviderCapabilities, SessionConfig, SpeechProvider};
use crate::session::{AudioSink, SpeechSession, TranscriptEvent};

/// Capacity of the transcript event channel. 32 is plenty — events fire at
/// most a few times per second per session.
const EVENT_CHANNEL_CAPACITY: usize = 32;

/// Capacity of the audio sink. Capture pushes ~50 frames/s; we don't consume
/// them in this provider (see module docs on `feeds_own_audio`) but we still
/// honor the trait by accepting frames into a draining task.
const SINK_CHANNEL_CAPACITY: usize = 64;

/// The Slice-1 speech backend: `Windows.Media.SpeechRecognition`.
///
/// This is the only provider that ships in the desktop app for slice 1.
/// Slice 2 adds whisper.cpp / faster-whisper, slice 4 adds cloud providers.
#[derive(Debug, Default)]
pub struct WindowsSpeechProvider;

impl WindowsSpeechProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SpeechProvider for WindowsSpeechProvider {
    fn id(&self) -> &'static str {
        "windows"
    }

    fn display_name(&self) -> &'static str {
        "Windows Speech Recognition"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming_partials: true,
            network_required: false,
            // Windows SR is tuned for normal-volume dictation; very quiet
            // ("whisper mode") speech is not in its design envelope.
            whisper_mode: false,
            // Single-language per session — must be set in SessionConfig.
            auto_language: false,
            gpu_capable: false,
            feeds_own_audio: true,
        }
    }

    async fn start_session(&self, config: SessionConfig) -> Result<SpeechSession, SpeechError> {
        let language = config
            .language
            .clone()
            .unwrap_or_else(|| "en-US".to_owned());
        let emit_partials = config.emit_partials;

        // Channel back to the caller carrying transcript events.
        let (events_tx, events_rx) = mpsc::channel::<TranscriptEvent>(EVENT_CHANNEL_CAPACITY);

        // Channel from the dropped session back to the recognizer thread.
        let (stop_tx, stop_rx) = oneshot::channel::<()>();

        // Audio sink: the provider feeds its own audio, but we still accept
        // and drain frames the orchestrator pushes — that keeps the trait
        // contract honest and avoids back-pressuring the audio pipeline.
        let (sink_tx, mut sink_rx) = mpsc::channel::<Vec<i16>>(SINK_CHANNEL_CAPACITY);
        tokio::spawn(async move {
            // Drain until the orchestrator drops its end. We don't do anything
            // with the audio — see module-level `feeds_own_audio` docs.
            while sink_rx.recv().await.is_some() {}
        });

        // Synchronously confirm the recognizer initialized before returning
        // the session. If we don't do this, the caller would see "session
        // started" and then a `TranscriptEvent::Error` arrive milliseconds
        // later — much harder to handle than a real `Result::Err`.
        let (init_tx, init_rx) = oneshot::channel::<Result<(), SpeechError>>();

        let events_tx_for_worker = events_tx.clone();
        std::thread::Builder::new()
            .name("contextflow-winsr".to_owned())
            .spawn(move || {
                recognizer_worker(
                    language,
                    emit_partials,
                    events_tx_for_worker,
                    init_tx,
                    stop_rx,
                );
            })
            .map_err(|e| SpeechError::InitFailed(format!("spawn recognizer worker: {e}")))?;

        // Wait for the worker to either confirm init or report failure.
        match init_rx.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_recv) => {
                return Err(SpeechError::InitFailed(
                    "recognizer worker dropped before initializing".to_owned(),
                ));
            }
        }

        info!("WindowsSpeechProvider session started");

        let session = WinSrSessionGuard {
            stop_tx: Some(stop_tx),
        };
        let stream = WinSrEventStream {
            inner: ReceiverStream::new(events_rx),
            _guard: Arc::new(session),
        }
        .boxed();

        Ok(SpeechSession {
            audio_sink: AudioSink::new(sink_tx),
            events: stream,
        })
    }
}

/// Drops the session → drops the stream → drops the guard → sends stop.
///
/// We attach the guard to the event stream rather than to `SpeechSession`
/// itself because consumers tend to hold the stream long after they've
/// stopped touching the session struct.
struct WinSrSessionGuard {
    stop_tx: Option<oneshot::Sender<()>>,
}

impl Drop for WinSrSessionGuard {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            // If send fails the worker already exited — fine either way.
            let _ = tx.send(());
        }
    }
}

/// Adapter so the boxed events stream keeps the stop-guard alive.
struct WinSrEventStream {
    inner: ReceiverStream<TranscriptEvent>,
    _guard: Arc<WinSrSessionGuard>,
}

impl futures::Stream for WinSrEventStream {
    type Item = TranscriptEvent;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// The dedicated recognizer thread. Owns the `SpeechRecognizer` and the COM
/// init for its lifetime.
fn recognizer_worker(
    language: String,
    emit_partials: bool,
    events_tx: mpsc::Sender<TranscriptEvent>,
    init_tx: oneshot::Sender<Result<(), SpeechError>>,
    stop_rx: oneshot::Receiver<()>,
) {
    // `Recognizer` and `ContinuousRecognitionSession` are WinRT objects; they
    // are RC'd via IInspectable internally. Calls into the runtime require
    // COM to be initialized on this thread.
    //
    // SAFETY: `CoInitializeEx` is sound to call from any thread once, paired
    // with `CoUninitialize` on the same thread. We balance them at exit below.
    let com_hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if com_hr.is_err() {
        let _ = init_tx.send(Err(SpeechError::InitFailed(format!(
            "CoInitializeEx(MTA): HRESULT {com_hr:?}"
        ))));
        return;
    }

    // Split init into a fallible setup phase and a run phase. Setup errors
    // come back through `init_tx` so `start_session` can return a real
    // `Result::Err`; run-phase errors (anything after StartAsync returned)
    // go through `events_tx` as `TranscriptEvent::Error`.
    match setup_recognizer(&language, emit_partials, &events_tx) {
        Err(e) => {
            warn!(?e, "windows recognizer setup failed");
            let _ = init_tx.send(Err(e));
        }
        Ok(handles) => {
            // Setup succeeded — release the caller, then run.
            if init_tx.send(Ok(())).is_ok() {
                let completed_flag = handles.completed_flag.clone();
                if let Err(e) = run_recognizer_loop(handles, completed_flag, stop_rx) {
                    warn!(?e, "windows recognizer run-phase error");
                    let _ = events_tx.try_send(TranscriptEvent::Error {
                        message: e.to_string(),
                        recoverable: e.is_recoverable(),
                    });
                }
            } else {
                // Caller dropped the future before we sent Ok — stop cleanly.
                let _ = handles.continuous.StopAsync().and_then(|op| op.get());
            }
        }
    }

    // SAFETY: paired with the CoInitializeEx above.
    unsafe { CoUninitialize() };
    debug!("contextflow-winsr thread exited");
}

/// Handles kept alive on the recognizer thread across setup → run.
///
/// `_recognizer` is held because the continuous session is borrowed from it;
/// dropping the recognizer would invalidate the session. `completed_flag`
/// is shared with the Completed event handler so the run loop knows when
/// the session has terminated on its own and StopAsync would error.
struct RecognizerHandles {
    _recognizer: SpeechRecognizer,
    continuous: SpeechContinuousRecognitionSession,
    completed_flag: Arc<std::sync::atomic::AtomicBool>,
}

/// Set up the recognizer through `StartAsync` and return live handles.
/// Any error here is an init-phase error — it goes back to `start_session`.
fn setup_recognizer(
    language: &str,
    emit_partials: bool,
    events_tx: &mpsc::Sender<TranscriptEvent>,
) -> Result<RecognizerHandles, SpeechError> {
    // SpeechRecognizer::Create takes a `Language`; constructing a Language
    // from an HSTRING (BCP-47 tag like "en-US") is the documented path.
    let language_obj =
        windows::Globalization::Language::CreateLanguage(&HSTRING::from(language))
            .map_err(|e| SpeechError::InitFailed(format!("language `{language}`: {e}")))?;

    let recognizer = SpeechRecognizer::Create(&language_obj)
        .map_err(|e| SpeechError::InitFailed(format!("SpeechRecognizer::Create: {e}")))?;

    // Default constraint: free dictation. CompileConstraintsAsync must be
    // called before StartAsync even if no constraints were added.
    let compile_op = recognizer
        .CompileConstraintsAsync()
        .map_err(|e| SpeechError::InitFailed(format!("CompileConstraintsAsync: {e}")))?;
    let compile_result = compile_op
        .get()
        .map_err(|e| SpeechError::InitFailed(format!("compile constraints get(): {e}")))?;
    let status = compile_result
        .Status()
        .map_err(|e| SpeechError::InitFailed(format!("compile result Status(): {e}")))?;
    if status != windows::Media::SpeechRecognition::SpeechRecognitionResultStatus::Success {
        return Err(SpeechError::InitFailed(format!(
            "constraint compile status = {status:?}"
        )));
    }

    let continuous: SpeechContinuousRecognitionSession = recognizer
        .ContinuousRecognitionSession()
        .map_err(|e| SpeechError::InitFailed(format!("ContinuousRecognitionSession: {e}")))?;

    // StateChanged: surface every recognizer state transition through tracing
    // (Idle → Capturing → SoundStarted → SpeechDetected → Processing → Idle).
    // Invaluable for diagnosing "session ran but no transcript" — tells us
    // whether WinRT ever saw any sound or speech at all.
    {
        let state_handler = TypedEventHandler::<
            SpeechRecognizer,
            SpeechRecognizerStateChangedEventArgs,
        >::new(move |_sender, args| {
            if let Some(args) = args.as_ref() {
                if let Ok(state) = args.State() {
                    info!(?state, "winsr state transition");
                }
            }
            Ok(())
        });
        recognizer
            .StateChanged(&state_handler)
            .map_err(|e| SpeechError::InitFailed(format!("StateChanged: {e}")))?;
    }

    // Partials (HypothesisGenerated): low-latency interim transcript.
    if emit_partials {
        let tx = events_tx.clone();
        let hypo_handler = TypedEventHandler::<
            SpeechRecognizer,
            SpeechRecognitionHypothesisGeneratedEventArgs,
        >::new(move |_sender, args| {
            if let Some(args) = args.as_ref() {
                if let Ok(hypo) = args.Hypothesis() {
                    if let Ok(text) = hypo.Text() {
                        let text = text.to_string_lossy();
                        if !text.trim().is_empty() {
                            // Hypothesis stability is not directly exposed;
                            // 0.5 is the convention for "interim, likely to change."
                            let _ = tx.try_send(TranscriptEvent::Partial {
                                text,
                                stability: 0.5,
                            });
                        }
                    }
                }
            }
            Ok(())
        });
        recognizer
            .HypothesisGenerated(&hypo_handler)
            .map_err(|e| SpeechError::InitFailed(format!("HypothesisGenerated: {e}")))?;
    }

    // Finals (ResultGenerated on the continuous session): committed transcript.
    {
        let tx = events_tx.clone();
        let result_handler = TypedEventHandler::<
            SpeechContinuousRecognitionSession,
            SpeechContinuousRecognitionResultGeneratedEventArgs,
        >::new(move |_sender, args| {
            if let Some(args) = args.as_ref() {
                if let Ok(result) = args.Result() {
                    let status = result
                        .Status()
                        .unwrap_or(SpeechRecognitionResultStatus::Unknown);
                    let text = result
                        .Text()
                        .map(|t| t.to_string_lossy())
                        .unwrap_or_default();
                    if status == SpeechRecognitionResultStatus::Success && !text.trim().is_empty() {
                        let _ = tx.try_send(TranscriptEvent::Final {
                            text,
                            alternatives: Vec::new(),
                        });
                    }
                }
            }
            Ok(())
        });
        continuous
            .ResultGenerated(&result_handler)
            .map_err(|e| SpeechError::InitFailed(format!("ResultGenerated: {e}")))?;
    }

    // Completed: fires on stop, timeout, or unrecoverable error. We also
    // flip the `completed` flag so the run loop knows not to call StopAsync
    // (it would just return E_INVALIDARG against an already-terminal session).
    let completed_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let tx = events_tx.clone();
        let flag = completed_flag.clone();
        let completed_handler = TypedEventHandler::<
            SpeechContinuousRecognitionSession,
            SpeechContinuousRecognitionCompletedEventArgs,
        >::new(move |_sender, args| {
            flag.store(true, std::sync::atomic::Ordering::Release);
            if let Some(args) = args.as_ref() {
                let status = args
                    .Status()
                    .unwrap_or(SpeechRecognitionResultStatus::Unknown);
                if status != SpeechRecognitionResultStatus::Success
                    && status != SpeechRecognitionResultStatus::UserCanceled
                {
                    let _ = tx.try_send(TranscriptEvent::Error {
                        message: describe_status(status),
                        recoverable: matches!(
                            status,
                            SpeechRecognitionResultStatus::TimeoutExceeded
                                | SpeechRecognitionResultStatus::PauseLimitExceeded
                                | SpeechRecognitionResultStatus::AudioQualityFailure
                        ),
                    });
                }
            }
            Ok(())
        });
        continuous
            .Completed(&completed_handler)
            .map_err(|e| SpeechError::InitFailed(format!("Completed: {e}")))?;
    }

    // Kick off recognition. The async op completes once the session is up.
    // Map known HRESULTs to actionable user-facing messages — this is the
    // single most failure-prone step on a fresh machine, so investing in a
    // good error here pays for itself.
    let start_op = continuous.StartAsync().map_err(|e| map_start_error(&e))?;
    start_op.get().map_err(|e| map_start_error(&e))?;

    Ok(RecognizerHandles {
        _recognizer: recognizer,
        continuous,
        completed_flag,
    })
}

/// Translate WinRT errors from `StartAsync` into messages the user can act on.
///
/// `0x80045509` (`SPERR_SPEECH_PRIVACY_POLICY_NOT_ACCEPTED`) is what fires the
/// first time anything on the machine uses `Windows.Media.SpeechRecognition`
/// without the user having toggled "Online speech recognition" in Settings.
#[allow(
    clippy::cast_possible_wrap,
    reason = "HRESULTs are conventionally written as u32 hex but compared as i32; \
              0x80045509 is intentionally the negative i32 SPERR_SPEECH_PRIVACY_POLICY_NOT_ACCEPTED."
)]
fn map_start_error(e: &windows::core::Error) -> SpeechError {
    const SPERR_PRIVACY_POLICY_NOT_ACCEPTED: i32 = 0x8004_5509_u32 as i32;
    if e.code().0 == SPERR_PRIVACY_POLICY_NOT_ACCEPTED {
        SpeechError::InitFailed(
            "Windows speech privacy policy not accepted. Open Settings → \
             Privacy & security → Speech, turn ON 'Online speech recognition' \
             (you can turn it back off after the first launch), then re-run."
                .to_owned(),
        )
    } else {
        SpeechError::InitFailed(format!("StartAsync: {e}"))
    }
}

/// Post-init loop: park the recognizer thread until the session is dropped
/// or the Completed event fires, then call `StopAsync` if appropriate.
fn run_recognizer_loop(
    handles: RecognizerHandles,
    completed_flag: Arc<std::sync::atomic::AtomicBool>,
    stop_rx: oneshot::Receiver<()>,
) -> Result<(), SpeechError> {
    // Block on the stop signal OR a Completed event from WinRT. We poll on
    // a short interval rather than using `blocking_recv` so we don't need
    // a tokio runtime on this thread.
    let mut stop_rx = stop_rx;
    loop {
        if completed_flag.load(std::sync::atomic::Ordering::Acquire) {
            // The session terminated itself (mic unavailable, timeout,
            // user cancel...). The Completed handler already pushed the
            // diagnostic event; calling StopAsync on a terminal session
            // returns E_INVALIDARG, so we just exit.
            return Ok(());
        }
        match stop_rx.try_recv() {
            Ok(()) | Err(oneshot::error::TryRecvError::Closed) => break,
            Err(oneshot::error::TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    // Graceful stop. `StopAsync` flushes the final result if one is pending.
    handles
        .continuous
        .StopAsync()
        .and_then(|op| op.get())
        .map_err(|e| SpeechError::SessionEnded(format!("StopAsync: {e}")))?;
    Ok(())
}

/// Translate WinRT result statuses into messages a user can act on.
fn describe_status(status: SpeechRecognitionResultStatus) -> String {
    // `SpeechRecognitionResultStatus` is a WinRT C-style enum; integer values
    // are stable across Windows versions. We match by name first and fall
    // back to a numeric form for variants the windows crate hasn't surfaced.
    match status {
        SpeechRecognitionResultStatus::Success => "success".to_owned(),
        SpeechRecognitionResultStatus::TopicLanguageNotSupported => {
            "topic language not supported — the recognizer doesn't have this language installed"
                .to_owned()
        }
        SpeechRecognitionResultStatus::GrammarLanguageMismatch => {
            "grammar language mismatch".to_owned()
        }
        SpeechRecognitionResultStatus::GrammarCompilationFailure => {
            "grammar compilation failure".to_owned()
        }
        SpeechRecognitionResultStatus::AudioQualityFailure => {
            "audio quality too low — try a quieter room or a closer mic".to_owned()
        }
        SpeechRecognitionResultStatus::UserCanceled => "user canceled".to_owned(),
        SpeechRecognitionResultStatus::Unknown => "unknown error".to_owned(),
        SpeechRecognitionResultStatus::TimeoutExceeded => {
            "timeout exceeded waiting for speech".to_owned()
        }
        SpeechRecognitionResultStatus::PauseLimitExceeded => {
            "pause limit exceeded — long silence ended the session".to_owned()
        }
        SpeechRecognitionResultStatus::NetworkFailure => "network failure".to_owned(),
        SpeechRecognitionResultStatus::MicrophoneUnavailable => {
            "microphone unavailable — open Settings → Privacy & security → Microphone, \
             ensure 'Microphone access', 'Let apps access your microphone' and \
             'Let desktop apps access your microphone' are all ON, then re-run"
                .to_owned()
        }
        other => format!("status {other:?}"),
    }
}
