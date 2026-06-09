//! Errors returned by text injectors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InjectionError {
    /// `SendInput` returned fewer events than we asked it to dispatch.
    ///
    /// This is what Windows reports when another low-level keyboard hook
    /// blocked the synthetic input — common with screen readers, some
    /// remote-desktop overlays, and security software. The orchestrator
    /// should surface this so the user knows their tool stack ate the
    /// injection rather than silently dropping the transcript.
    #[error(
        "SendInput dispatched only {dispatched} of {requested} events — likely blocked by a \
         low-level keyboard hook (screen reader / RDP / security tool)"
    )]
    HookBlocked { requested: u32, dispatched: u32 },

    /// `SendInput` returned 0 events with a non-success last-error code.
    #[error("SendInput failed: {0}")]
    Win32(String),

    /// The text we were asked to inject is empty — caller bug, not a runtime
    /// failure. Treated as `Ok` by `SendInputInjector::inject`; this variant
    /// exists so future strategies (UIA, clipboard) can opt to reject it.
    #[error("nothing to inject")]
    Empty,
}
