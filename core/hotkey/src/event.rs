//! The typed hotkey event flowing through the bus.

use serde::{Deserialize, Serialize};

/// A single observable hotkey transition.
///
/// Hold-to-talk semantics: `Pressed` opens a dictation session, `Released`
/// closes it. Tap-to-toggle (a configurable variant) reuses the same enum
/// but with a `mode` flag — added in Slice 5 when settings ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HotkeyEvent {
    /// The configured push-to-talk hotkey went down.
    Pressed,
    /// The configured push-to-talk hotkey went up.
    Released,
}

impl HotkeyEvent {
    #[must_use]
    pub fn is_pressed(self) -> bool {
        matches!(self, Self::Pressed)
    }
}
