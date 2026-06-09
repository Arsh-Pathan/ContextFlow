use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::SpeechError;
use crate::provider::{ProviderCapabilities, SessionConfig, SpeechProvider};
use crate::session::{AudioSink, SpeechSession, TranscriptEvent};

pub struct WhisperCppProvider {
    context: Arc<WhisperContext>,
}

impl WhisperCppProvider {
    pub fn new(model_path: PathBuf) -> Result<Self, SpeechError> {
        let params = WhisperContextParameters::default();
        let context = WhisperContext::new_with_params(&model_path.to_string_lossy(), params)
            .map_err(|e| SpeechError::InitFailed(format!("Failed to load Whisper model: {}", e)))?;
        
        Ok(Self {
            context: Arc::new(context),
        })
    }
}

#[async_trait]
impl SpeechProvider for WhisperCppProvider {
    fn id(&self) -> &'static str {
        "whisper-cpp"
    }

    fn display_name(&self) -> &'static str {
        "Whisper (Local)"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming_partials: false, // Whisper processes full audio at the end
            network_required: false,
            whisper_mode: true,
            auto_language: true,
            gpu_capable: true, // depends on whisper.cpp build
            feeds_own_audio: false, // consumes audio from our pipeline
        }
    }

    async fn start_session(&self, config: SessionConfig) -> Result<SpeechSession, SpeechError> {
        let (sink_tx, mut sink_rx) = mpsc::channel::<Vec<i16>>(4096);
        let (events_tx, events_rx) = mpsc::channel::<TranscriptEvent>(32);

        let context = self.context.clone();
        let language = config.language.clone();

        std::thread::Builder::new()
            .name("contextflow-whisper".to_string())
            .spawn(move || {
                let mut audio_buffer = Vec::new();

                // Block and wait for the orchestrator to send audio.
                // The loop breaks when the orchestrator drops the audio_sink (hotkey released).
                while let Some(mut frame) = sink_rx.blocking_recv() {
                    audio_buffer.append(&mut frame);
                }
                
                if audio_buffer.is_empty() {
                    let _ = events_tx.blocking_send(TranscriptEvent::Empty);
                    return;
                }

                debug!("utterance ended, running whisper inference on {} samples", audio_buffer.len());

                // whisper-rs expects f32 samples between -1 and 1
                let audio_f32: Vec<f32> = audio_buffer.into_iter()
                    .map(|s| s as f32 / 32768.0)
                    .collect();

                let mut state = match context.create_state() {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = events_tx.blocking_send(TranscriptEvent::Error {
                            message: format!("create_state failed: {e}"),
                            recoverable: false,
                        });
                        return;
                    }
                };

                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                params.set_print_progress(false);
                params.set_print_special(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);
                
                if let Some(ref lang) = language {
                    params.set_language(Some(lang.as_str()));
                } else {
                    params.set_language(Some("en"));
                }

                if let Err(e) = state.full(params, &audio_f32) {
                    let _ = events_tx.blocking_send(TranscriptEvent::Error {
                        message: format!("inference failed: {e}"),
                        recoverable: false,
                    });
                    return;
                }

                let num_segments = match state.full_n_segments() {
                    Ok(n) => n,
                    Err(_) => 0,
                };

                let mut final_text = String::new();
                for i in 0..num_segments {
                    if let Ok(segment) = state.full_get_segment_text(i) {
                        final_text.push_str(&segment);
                    }
                }

                let final_text = final_text.trim().to_string();
                
                if final_text.is_empty() {
                    let _ = events_tx.blocking_send(TranscriptEvent::Empty);
                } else {
                    let _ = events_tx.blocking_send(TranscriptEvent::Final {
                        text: final_text,
                        alternatives: vec![],
                    });
                }
            })
            .map_err(|e| SpeechError::InitFailed(format!("failed to spawn thread: {e}")))?;

        Ok(SpeechSession {
            audio_sink: AudioSink::new(sink_tx),
            events: Box::pin(ReceiverStream::new(events_rx)),
        })
    }
}
