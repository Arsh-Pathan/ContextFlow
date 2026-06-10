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
///
/// The optional fields are populated by the orchestrator based on the
/// state being entered:
///
/// * `level` is set when `status == Listening` so the bubble can pulse
///   with the live RMS level the audio engine is reporting. Range
///   `0.0..=1.0`.
/// * `message` is set when `status == Error` to carry a one-line user-
///   facing description ("microphone unavailable", "speech privacy not
///   accepted", etc.) for the bubble's tooltip and the in-app log.
/// * `provider` is set once at startup to show which speech engine is
///   active (e.g. "whisper-cpp" or "windows-sr").
/// * `warning` can be set alongside any status to carry a non-fatal
///   advisory ("Falling back to Windows SR because whisper model
///   failed to load").
///
/// All optional fields are `None` when they don't apply. The UI MUST
/// tolerate any of them being absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationStatusEvent {
    pub status: DictationStatus,

    /// Live audio RMS in `0.0..=1.0`. Populated during `Listening`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub level: Option<f32>,

    /// User-facing error context. Populated during `Error`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,

    /// Provider identifier shown in the bubble tooltip.
    /// Set once at startup (e.g. "whisper-cpp", "windows-sr").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<String>,

    /// Non-fatal warning shown alongside any status.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub warning: Option<String>,
}

impl DictationStatusEvent {
    #[must_use]
    pub fn new(status: DictationStatus) -> Self {
        Self {
            status,
            level: None,
            message: None,
            provider: None,
            warning: None,
        }
    }

    /// Convenience: `Listening` event carrying the latest RMS level.
    #[must_use]
    pub fn listening(level: f32) -> Self {
        Self {
            status: DictationStatus::Listening,
            level: Some(level),
            message: None,
            provider: None,
            warning: None,
        }
    }

    /// Convenience: `Error` event carrying a user-facing message.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: DictationStatus::Error,
            level: None,
            message: Some(message.into()),
            provider: None,
            warning: None,
        }
    }

    /// Attach the provider identifier to the event.
    #[must_use]
    pub fn with_provider(mut self, provider: &'static str) -> Self {
        self.provider = Some(provider.to_owned());
        self
    }

    /// Attach a non-fatal warning to the event.
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warning = Some(warning.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_snake_case() {
        let json =
            serde_json::to_string(&DictationStatusEvent::new(DictationStatus::Listening)).unwrap();
        // Optional fields are omitted when None — keeps the wire small and
        // the JavaScript-side type guards simple.
        assert_eq!(json, r#"{"status":"listening"}"#);
    }

    #[test]
    fn listening_event_carries_level() {
        let json = serde_json::to_string(&DictationStatusEvent::listening(0.42)).unwrap();
        // f32 serialisation in serde_json uses the shortest round-trippable
        // representation; pinning the exact string would be brittle. Check
        // the fields are present and the level round-trips through parse.
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["status"], "listening");
        let level = parsed["level"].as_f64().unwrap();
        assert!((level - 0.42).abs() < 1e-6, "got {level}");
        assert!(parsed.get("message").is_none());
    }

    #[test]
    fn error_event_carries_message() {
        let json =
            serde_json::to_string(&DictationStatusEvent::error("microphone unavailable")).unwrap();
        assert_eq!(
            json,
            r#"{"status":"error","message":"microphone unavailable"}"#
        );
    }

    #[test]
    fn extra_fields_round_trip() {
        // Forward-compat: a UI built against an older schema must still
        // accept a richer payload. Round-trip a fully-populated event and
        // verify nothing is dropped.
        let original = DictationStatusEvent {
            status: DictationStatus::Listening,
            level: Some(0.75),
            message: Some("captured".to_owned()),
            provider: Some("whisper".to_owned()),
            warning: Some("low battery".to_owned()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: DictationStatusEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, DictationStatus::Listening);
        assert_eq!(back.level, Some(0.75));
        assert_eq!(back.message.as_deref(), Some("captured"));
        assert_eq!(back.provider.as_deref(), Some("whisper"));
        assert_eq!(back.warning.as_deref(), Some("low battery"));
    }
}
