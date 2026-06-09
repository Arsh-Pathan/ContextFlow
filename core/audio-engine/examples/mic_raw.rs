//! Raw-input diagnostic. Bypasses our downmix/resample/VAD pipeline and
//! prints the peak absolute sample value coming straight out of cpal for
//! each callback. Use this to tell whether silence is coming from Windows
//! (mic muted, permission denied, wrong endpoint) or from our pipeline.
//!
//! Run with:
//!
//!   cargo run -p contextflow-audio-engine --example mic_raw
//!
//! Expected when the mic works:
//!
//!   device  = "Microphone Array (...)"
//!   rate    = 48000  channels = 2  format = F32
//!   callback peak = 0.018   (any non-zero value while you speak)
//!   callback peak = 0.142
//!   ...
//!
//! Expected when Windows is delivering silence (muted, permission denied,
//! wrong device):
//!
//!   callback peak = 0.000
//!   callback peak = 0.000
//!   ...
//!
//! Also enumerates every input device the host can see so we can confirm
//! we're opening the one the user expects.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    println!("─── input devices ───");
    match host.input_devices() {
        Ok(it) => {
            for (i, dev) in it.enumerate() {
                let name = dev.name().unwrap_or_else(|_| "<unknown>".to_owned());
                let cfg = dev.default_input_config().map_or_else(
                    |e| format!("<no default config: {e}>"),
                    |c| {
                        format!(
                            "{} ch, {} Hz, {:?}",
                            c.channels(),
                            c.sample_rate().0,
                            c.sample_format()
                        )
                    },
                );
                println!("  [{i}] {name}  ({cfg})");
            }
        }
        Err(e) => println!("  (could not enumerate: {e})"),
    }
    println!();

    let device = host
        .default_input_device()
        .ok_or("no default input device")?;
    let name = device.name().unwrap_or_else(|_| "<unknown>".to_owned());

    let default_config = device.default_input_config()?;
    let sample_format = default_config.sample_format();
    let config = default_config.into();
    println!("opening default input: {name}");
    println!("  config = {config:?}  format = {sample_format:?}");

    // Track peak (×1000 so we can store in an AtomicU32).
    let peak_milli = Arc::new(AtomicU32::new(0));
    let total_samples = Arc::new(AtomicU32::new(0));

    let stream = match sample_format {
        SampleFormat::F32 => {
            let peak = peak_milli.clone();
            let total = total_samples.clone();
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mut local_peak = 0.0_f32;
                    for &s in data {
                        let a = s.abs();
                        if a > local_peak {
                            local_peak = a;
                        }
                    }
                    let scaled = (local_peak * 1000.0).round() as u32;
                    peak.fetch_max(scaled, Ordering::Relaxed);
                    total.fetch_add(data.len() as u32, Ordering::Relaxed);
                },
                |err| eprintln!("stream error: {err}"),
                None,
            )?
        }
        SampleFormat::I16 => {
            let peak = peak_milli.clone();
            let total = total_samples.clone();
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let mut local_peak = 0.0_f32;
                    for &s in data {
                        let a = (f32::from(s) / f32::from(i16::MAX)).abs();
                        if a > local_peak {
                            local_peak = a;
                        }
                    }
                    peak.fetch_max((local_peak * 1000.0).round() as u32, Ordering::Relaxed);
                    total.fetch_add(data.len() as u32, Ordering::Relaxed);
                },
                |err| eprintln!("stream error: {err}"),
                None,
            )?
        }
        SampleFormat::U16 => {
            let peak = peak_milli.clone();
            let total = total_samples.clone();
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let mut local_peak = 0.0_f32;
                    for &s in data {
                        let centered = i32::from(s) - 32_768;
                        let a = (centered as f32 / 32_768.0).abs();
                        if a > local_peak {
                            local_peak = a;
                        }
                    }
                    peak.fetch_max((local_peak * 1000.0).round() as u32, Ordering::Relaxed);
                    total.fetch_add(data.len() as u32, Ordering::Relaxed);
                },
                |err| eprintln!("stream error: {err}"),
                None,
            )?
        }
        other => return Err(format!("unsupported sample format {other:?}").into()),
    };

    stream.play()?;
    println!();
    println!("Speak now. Sampling raw cpal callbacks for 5 seconds...");

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(250));
        let p = peak_milli.swap(0, Ordering::Relaxed);
        println!("  peak (last 250 ms) = {:.3}", f64::from(p) / 1000.0);
    }

    drop(stream);

    let total = total_samples.load(Ordering::Relaxed);
    println!();
    println!("Total raw samples observed: {total}");
    if total == 0 {
        println!("⚠ cpal delivered zero callbacks. The stream may have failed to start.");
    } else {
        println!("✓ cpal delivered {total} samples over ~5 s.");
        println!();
        println!("If every 'peak' above is 0.000 then Windows is handing us silence:");
        println!("  1. Windows Settings → Privacy & security → Microphone");
        println!("     • 'Microphone access' = On");
        println!("     • 'Let desktop apps access your microphone' = On");
        println!("  2. Windows Settings → System → Sound → Input");
        println!("     • Speak into mic; the blue level bar must move.");
        println!("     • If it doesn't, the OS itself isn't getting audio (mute / driver).");
        println!("  3. If a specific device above looks more correct than the default,");
        println!("     change the Windows default input device and re-run.");
    }
    Ok(())
}
