//! ContextFlow text injection.
//!
//! Inserts text into whatever window currently has keyboard focus on
//! Windows. The trait is [`TextInjector`]; the slice-1 implementation is
//! [`SendInputInjector`], which synthesizes Unicode keystrokes via the
//! Win32 `SendInput` API with `KEYEVENTF_UNICODE`.
//!
//! ## Why `SendInput` for slice 1
//!
//! It's the lowest-common-denominator path that just works in any normal
//! text field — Notepad, Chrome address bar, VS Code, Word, Discord (mostly),
//! the Run dialog. No COM, no UI Automation, no clipboard side effects, no
//! per-app code paths. Per the slice-1 engineering rule ("fastest path to
//! proving the product works"), this is what we ship first.
//!
//! Slice 3 layers on top:
//!
//! * UI Automation (`IUIAutomationTextPattern`) as the primary path, so
//!   text appears as a single revision and respects the host app's
//!   composition behavior (Word, modern WPF/WinUI text boxes, Outlook).
//! * Clipboard paste with save / inject / restore as the last-resort
//!   fallback for fields that refuse keystrokes (some Electron apps).
//! * Per-app strategy routing (Chrome, VS Code, Cursor, Slack, Discord,
//!   Word, Outlook, Teams).
//!
//! ## Threading
//!
//! `SendInput` is a synchronous Win32 call. We invoke it from a
//! `tokio::task::spawn_blocking` so the async runtime is not stalled,
//! even though each call returns in well under a millisecond on a
//! typical machine.

pub mod error;
pub mod injector;
pub mod send_input;

pub use error::InjectionError;
pub use injector::TextInjector;
pub use send_input::SendInputInjector;
