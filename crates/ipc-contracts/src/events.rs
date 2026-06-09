//! Tauri events emitted from the Rust shell to the React UI.

use serde::{Deserialize, Serialize};

/// Topic name for the dictation status event, used by both `app.emit(...)` on
/// the Rust side and `listen(...)` on the React side. Centralised here so we
/// never have a string-typo mismatch between the two.
pub const EVENT_DICTATION_STATUS: &str = "dictation://status";

/// One of the four visible states the floating bubble can show.
///
/// Mirrors the React `DictationStatus` type in `apps/desktop/src/components/Bubble.tsx`.
/// Keep the two in sync — a binding generator lands in a follow-up commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationStatus {
    Idle,
    Listening,
    Processing,
    Error,
}

impl DictationStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Processing => "processing",
            Self::Error => "error",
        }
    }
}

/// Payload sent on [`EVENT_DICTATION_STATUS`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationStatusEvent {
    pub status: DictationStatus,
}

impl DictationStatusEvent {
    #[must_use]
    pub fn new(status: DictationStatus) -> Self {
        Self { status }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_snake_case() {
        let json = serde_json::to_string(&DictationStatusEvent::new(DictationStatus::Listening))
            .unwrap();
        assert_eq!(json, r#"{"status":"listening"}"#);
    }
}
