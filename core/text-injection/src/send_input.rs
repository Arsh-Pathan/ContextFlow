//! `SendInput` Unicode keystroke injector — the slice-1 backend.
//!
//! Synthesizes a key-down + key-up event pair per UTF-16 code unit using
//! `SendInput` with `KEYEVENTF_UNICODE`. This works in any window that
//! accepts keyboard input the normal way — text fields, address bars,
//! editors, terminals — and in practice covers the slice-1 acceptance
//! test (dictate → text appears in Notepad).
//!
//! ## Threading
//!
//! `SendInput` is synchronous Win32. We hop to a `spawn_blocking` thread
//! so the async runtime is never parked on a kernel call, even though the
//! call itself returns in well under a millisecond for normal-length
//! transcripts.
//!
//! ## Surrogate pairs and emoji
//!
//! `KEYEVENTF_UNICODE` takes a UTF-16 code unit in `wScan`, not a Unicode
//! scalar value. Characters in the Basic Multilingual Plane (BMP) are one
//! code unit; anything outside (most emoji, supplementary CJK ideographs)
//! is a surrogate pair — two events for one `char`. We let
//! [`str::encode_utf16`] do the conversion, then emit one down/up pair
//! per code unit. Windows reassembles the surrogate pair into the right
//! `char` in the focused field on its own.
//!
//! ## Why not just `SendKeys`-style virtual keys?
//!
//! `wVk` only covers ASCII-ish keys on the active layout. The user's
//! transcript can be anything (accented characters from auto-correct,
//! curly quotes, em-dashes, emoji). `KEYEVENTF_UNICODE` is layout-
//! independent — exactly what we want for dictation.
//!
//! ## Chunking
//!
//! `SendInput`'s upper bound on a single call is "a lot" (~4 MB / event
//! struct size = ~100k events), but the kernel's keyboard buffer is
//! smaller and the perceived behavior of bursts > ~1024 events varies by
//! app. We chunk at 256 events (128 chars BMP / 64 chars surrogate) per
//! call; for normal dictation utterances that's one or two calls total.

use std::mem::size_of;

use async_trait::async_trait;
use tracing::{debug, trace, warn};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY,
};

use crate::error::InjectionError;
use crate::injector::{InjectorKind, TextInjector};

/// Maximum number of `INPUT` structs per `SendInput` call. See module docs.
const SEND_INPUT_CHUNK: usize = 256;

/// `SendInput` Unicode injector. Stateless — share one across the app.
#[derive(Debug, Default, Clone, Copy)]
pub struct SendInputInjector;

impl SendInputInjector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TextInjector for SendInputInjector {
    fn kind(&self) -> InjectorKind {
        InjectorKind::SendInput
    }

    async fn inject(&self, text: &str) -> Result<(), InjectionError> {
        if text.is_empty() {
            return Ok(());
        }
        // Copy the string into the blocking task — SendInput won't touch it
        // after the call returns, and we don't want to borrow across .await.
        let owned = text.to_owned();
        tokio::task::spawn_blocking(move || inject_blocking(&owned))
            .await
            .map_err(|e| InjectionError::Win32(format!("spawn_blocking: {e}")))?
    }
}

/// Build INPUT structs for each UTF-16 code unit and dispatch in chunks.
fn inject_blocking(text: &str) -> Result<(), InjectionError> {
    let inputs = build_unicode_inputs(text);
    if inputs.is_empty() {
        return Ok(());
    }

    let cb_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| InjectionError::Win32("INPUT struct size doesn't fit in i32".to_owned()))?;

    for chunk in inputs.chunks(SEND_INPUT_CHUNK) {
        // SAFETY: `SendInput` reads exactly `chunk.len()` `INPUT` structs from
        // the slice. The slice is valid for the duration of the call, the
        // INPUT layout matches what the kernel expects (we built them via
        // the windows crate's bindings), and `cb_size` is the size of one
        // INPUT struct as the API requires.
        let dispatched = unsafe { SendInput(chunk, cb_size) };
        let requested = u32::try_from(chunk.len()).unwrap_or(u32::MAX);
        if dispatched != requested {
            // Convention: SendInput returns the number of events successfully
            // dispatched. Anything less than requested means a low-level hook
            // ate the rest — almost always a screen reader, RDP overlay, or
            // security tool. Surface it so the orchestrator can show a
            // useful error rather than silently dropping the transcript.
            warn!(
                requested,
                dispatched, "SendInput dispatched fewer events than requested"
            );
            return Err(InjectionError::HookBlocked {
                requested,
                dispatched,
            });
        }
        trace!(events = requested, "SendInput chunk dispatched");
    }
    debug!(chars = text.chars().count(), "text injection complete");
    Ok(())
}

/// Map a `&str` to alternating keydown / keyup INPUT events, one pair per
/// UTF-16 code unit. Surrogate pairs become two pairs (4 events) — that's
/// what `KEYEVENTF_UNICODE` expects.
fn build_unicode_inputs(text: &str) -> Vec<INPUT> {
    // 2 INPUT structs per UTF-16 code unit (down + up). A surrogate pair
    // character contributes 4. encode_utf16 doesn't expose len up-front so
    // we estimate at 2x the char count and let the Vec grow if needed.
    let mut inputs = Vec::with_capacity(text.chars().count() * 2);
    for code_unit in text.encode_utf16() {
        inputs.push(unicode_input(code_unit, /* key_up */ false));
        inputs.push(unicode_input(code_unit, /* key_up */ true));
    }
    inputs
}

/// Build a single Unicode INPUT event. `wVk` must be 0 when
/// `KEYEVENTF_UNICODE` is set; `wScan` carries the UTF-16 code unit.
fn unicode_input(code_unit: u16, key_up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: code_unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_produces_no_inputs() {
        let inputs = build_unicode_inputs("");
        assert!(inputs.is_empty());
    }

    #[test]
    fn ascii_char_produces_two_inputs() {
        let inputs = build_unicode_inputs("A");
        assert_eq!(inputs.len(), 2);
    }

    #[test]
    fn surrogate_pair_produces_four_inputs() {
        // U+1F600 GRINNING FACE — outside the BMP, encodes to a UTF-16
        // surrogate pair, so we expect 4 INPUT events (down/up × 2 units).
        let inputs = build_unicode_inputs("\u{1F600}");
        assert_eq!(inputs.len(), 4);
    }

    #[test]
    fn count_matches_utf16_code_units_doubled() {
        let s = "Hello, world! 你好 🎉";
        let expected = s.encode_utf16().count() * 2;
        assert_eq!(build_unicode_inputs(s).len(), expected);
    }
}
