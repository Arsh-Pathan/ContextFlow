//! How a hotkey is described in settings and on the wire.
//!
//! Slice 1 hard-codes `Ctrl+Space`. Slice 5 (settings UI) lets the user
//! reconfigure this; the same struct round-trips through the settings
//! store and the global-shortcut registration call.

use serde::{Deserialize, Serialize};

/// A keyboard chord. Format matches Tauri's `tauri-plugin-global-shortcut`
/// accelerator syntax (e.g. `"Ctrl+Space"`, `"Ctrl+Shift+D"`).
///
/// We don't parse the string here — we hand it straight to the platform layer.
/// Parsing would just be a second source of truth that drifts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub accelerator: String,
}

impl HotkeyBinding {
    pub fn new(accelerator: impl Into<String>) -> Self {
        Self {
            accelerator: accelerator.into(),
        }
    }

    /// The default push-to-talk binding shipped in Slice 1.
    #[must_use]
    pub fn default_ptt() -> Self {
        Self::new("Ctrl+Space")
    }
}

impl Default for HotkeyBinding {
    fn default() -> Self {
        Self::default_ptt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ptt_is_ctrl_space() {
        assert_eq!(HotkeyBinding::default().accelerator, "Ctrl+Space");
    }

    #[test]
    fn round_trips_through_json() {
        let b = HotkeyBinding::new("Ctrl+Alt+D");
        let s = serde_json::to_string(&b).unwrap();
        let r: HotkeyBinding = serde_json::from_str(&s).unwrap();
        assert_eq!(b, r);
    }
}
