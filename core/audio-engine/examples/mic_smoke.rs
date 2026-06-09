//! Smoke test for the audio engine: opens the default mic, captures for a
//! few seconds, prints RMS levels and voiced/silent frame counts.
//!
//! Run with:
//!
//!   cargo run -p contextflow-audio-engine --example mic_smoke
//!
//! Hold a normal-volume conversation (or play music) for the duration —
//! you should see RMS values bouncing and the voiced/silent counts changing.
//! If RMS stays at 0.000 the wrong device is selected; check Windows Sound
//! settings → Input.

use std::time::Duration;

use contextflow_audio_engine::AudioEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    contextflow_telemetry::install_dev_subscriber();

    println!("ContextFlow audio smoke test — opening default input device...");
    let engine = AudioEngine::start()?;
    let mut frames = engine.subscribe_frames();
    let mut levels = engine.subscribe_levels();

    let stop = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut voiced = 0u64;
    let mut silent = 0u64;
    let mut last_level_print = tokio::time::Instant::now();

    println!("Speak now. Capturing 5 seconds...");
    while tokio::time::Instant::now() < stop {
        tokio::select! {
            ev = frames.recv() => {
                match ev {
                    Ok(f) => {
                        if f.voiced { voiced += 1; } else { silent += 1; }
                    }
                    Err(e) => {
                        eprintln!("frame recv error: {e}");
                        break;
                    }
                }
            }
            ev = levels.recv() => {
                if let Ok(rms) = ev {
                    if last_level_print.elapsed() > Duration::from_millis(200) {
                        let bar = bar_for(rms);
                        println!("rms = {rms:.3}  {bar}");
                        last_level_print = tokio::time::Instant::now();
                    }
                }
            }
        }
    }

    println!();
    println!("Captured {voiced} voiced frames, {silent} silent frames.");
    if voiced + silent == 0 {
        println!("⚠ No frames produced — is the worker thread running?");
    } else if voiced == 0 {
        println!("⚠ No voiced frames — try speaking louder, or check the input device.");
    } else {
        println!("✓ Audio capture is working.");
    }

    Ok(())
}

fn bar_for(rms: f32) -> String {
    let width = (rms * 40.0).clamp(0.0, 40.0) as usize;
    let mut s = String::with_capacity(40);
    for _ in 0..width {
        s.push('█');
    }
    s
}
