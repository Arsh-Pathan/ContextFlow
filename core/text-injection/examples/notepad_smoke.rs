//! Smoke test for `SendInputInjector`: types a fixed sentence (plus a
//! BMP-outside character, to exercise the surrogate-pair code path) into
//! whatever window currently has keyboard focus.
//!
//! Run with:
//!
//!   cargo run -p contextflow-text-injection --example notepad_smoke
//!
//! Steps:
//!
//!   1. Open Notepad (or any text field — Chrome address bar, VS Code…).
//!   2. Click into it so it has keyboard focus.
//!   3. From a *separate* PowerShell window, run the command above.
//!   4. You have ~5 seconds before the injection fires. Use that time to
//!      click back into the target field — focus it just before the
//!      countdown ends.
//!
//! Expected: the focused field receives
//!
//!   ContextFlow slice 1 — text injection works. 🎉
//!
//! If you see nothing, the most common cause is that the source PowerShell
//! window stole focus back. Re-run and click into the target field after
//! pressing Enter on the cargo command.

use std::time::Duration;

use contextflow_text_injection::{SendInputInjector, TextInjector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    contextflow_telemetry::install_dev_subscriber();

    let message = "ContextFlow slice 1 \u{2014} text injection works. \u{1F389}";

    println!("ContextFlow text-injection smoke test");
    println!("In 5 seconds I will type this into the focused window:");
    println!("    {message}");
    println!();
    for n in (1..=5).rev() {
        println!("  injecting in {n}...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let injector = SendInputInjector::new();
    injector.inject(message).await?;

    println!();
    println!("\u{2713} SendInput dispatched. Check the focused window.");
    Ok(())
}
