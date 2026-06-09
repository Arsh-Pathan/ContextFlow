//! Smoke test for `WindowsSpeechProvider`: start a session, listen for
//! 10 seconds, print every partial and final transcript that arrives.
//!
//! Run with:
//!
//!   cargo run -p contextflow-speech-engine --features provider-windows \
//!     --example windows_sr_smoke
//!
//! Then speak normal-volume English at your microphone. You should see
//! lines like:
//!
//!   [partial] hello world
//!   [partial] hello world this is
//!   [partial] hello world this is a test
//!   [FINAL ] hello world this is a test
//!
//! If you see no partials at all, the most common cause is that the
//! Windows speech language pack is not installed for en-US — open
//! Settings → Time & Language → Language & region → English (United
//! States) → "Speech" feature → Install.

use std::time::Duration;

use futures::StreamExt;

use contextflow_speech_engine::{
    providers::windows_sr::WindowsSpeechProvider, SessionConfig, SpeechProvider, TranscriptEvent,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    contextflow_telemetry::install_dev_subscriber();

    println!("ContextFlow speech smoke test — Windows.Media.SpeechRecognition");
    println!("Starting session (this opens the system microphone)...");
    let provider = WindowsSpeechProvider::new();
    let session = provider
        .start_session(SessionConfig::default())
        .await
        .map_err(|e| {
            eprintln!("could not start session: {e}");
            e
        })?;

    println!("Session ready. Speak normally for ~10 seconds; partials and the");
    println!("final transcript will be printed below.");
    println!();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut events = session.events;
    let mut got_anything = false;
    loop {
        tokio::select! {
            ev = events.next() => {
                match ev {
                    Some(TranscriptEvent::Partial { text, stability }) => {
                        got_anything = true;
                        println!("[partial s={stability:.2}] {text}");
                    }
                    Some(TranscriptEvent::Final { text, .. }) => {
                        got_anything = true;
                        println!("[FINAL ] {text}");
                    }
                    Some(TranscriptEvent::Empty) => {
                        println!("[empty ]");
                    }
                    Some(TranscriptEvent::Error { message, recoverable }) => {
                        eprintln!("[ERROR recoverable={recoverable}] {message}");
                    }
                    None => {
                        println!("(event stream ended)");
                        break;
                    }
                }
            }
            () = tokio::time::sleep_until(deadline) => {
                println!();
                println!("10 seconds elapsed — dropping session.");
                break;
            }
        }
    }

    if got_anything {
        println!("✓ Windows Speech Recognition is producing transcripts.");
    } else {
        println!();
        println!("⚠ No transcript events arrived. Likely cause: en-US speech");
        println!("  feature is not installed. Settings → Time & Language →");
        println!("  Language & region → English (United States) → Speech →");
        println!("  Install. Then re-run.");
    }

    // Dropping `events` (and through it the session guard) sends StopAsync.
    drop(events);
    // Give the WinRT thread a beat to call StopAsync and CoUninitialize.
    tokio::time::sleep(Duration::from_millis(250)).await;
    Ok(())
}
