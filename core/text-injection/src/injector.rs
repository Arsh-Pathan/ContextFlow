//! The [`TextInjector`] trait.
//!
//! Every text-injection backend implements this. Slice 1 ships only
//! [`crate::SendInputInjector`]; slice 3 adds UIA and clipboard strategies
//! and a `RoutingInjector` that picks one per focused app.

use async_trait::async_trait;

use crate::error::InjectionError;

/// Stable identifier for a text-injection strategy, used in logs and
/// the per-app routing table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectorKind {
    /// Win32 `SendInput` with `KEYEVENTF_UNICODE`.
    SendInput,
    /// `IUIAutomationTextPattern` (slice 3).
    Uia,
    /// Save / set / paste / restore against the system clipboard (slice 3).
    Clipboard,
}

impl InjectorKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SendInput => "send_input",
            Self::Uia => "uia",
            Self::Clipboard => "clipboard",
        }
    }
}

/// Inserts text into the currently focused window.
///
/// `inject` is `async` because slice 3's UIA path needs to talk COM, which
/// we run on a dedicated thread and await across. Slice 1's `SendInput`
/// path is synchronous-ish (one `spawn_blocking` and back).
#[async_trait]
pub trait TextInjector: Send + Sync {
    /// The strategy this injector implements.
    fn kind(&self) -> InjectorKind;

    /// Insert `text` into the focused window. Empty input is a no-op.
    async fn inject(&self, text: &str) -> Result<(), InjectionError>;
}
