//! The async event bus that carries [`HotkeyEvent`]s from the OS hook to the
//! dictation orchestrator.
//!
//! We use a broadcast channel rather than mpsc so multiple consumers can
//! subscribe — the dictation engine takes one receiver, the bubble UI's
//! Tauri-side bridge takes another. Late subscribers will miss events from
//! before they subscribed; that's fine for hold-to-talk because the state of
//! interest is the *next* press, not the past.

use tokio::sync::broadcast;

use crate::event::HotkeyEvent;

/// Wrapped receiver side; consumers call `recv().await`.
pub type HotkeyReceiver = broadcast::Receiver<HotkeyEvent>;

/// Wrapped sender side; the platform hook calls `send`.
#[derive(Debug, Clone)]
pub struct HotkeySender {
    inner: broadcast::Sender<HotkeyEvent>,
}

impl HotkeySender {
    /// Publish an event. If there are no subscribers we log and drop —
    /// hotkey events are transient and not worth queuing.
    pub fn send(&self, event: HotkeyEvent) {
        if let Err(err) = self.inner.send(event) {
            // `SendError::no_receivers` is normal during startup before the
            // orchestrator subscribes; log at trace so it doesn't spam.
            tracing::trace!(?err, "hotkey event dropped (no receivers)");
        }
    }
}

/// Owns a broadcast channel for hotkey events.
///
/// Construct one at startup, hand `sender()` to the platform integration
/// (Tauri shell), and `subscribe()` to each consumer.
#[derive(Debug, Clone)]
pub struct HotkeyBus {
    sender: broadcast::Sender<HotkeyEvent>,
}

impl HotkeyBus {
    /// Create a bus with the given capacity. 16 is plenty — hotkey events
    /// are at the speed of fingers, not packets.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _initial_rx) = broadcast::channel(capacity);
        Self { sender }
    }

    #[must_use]
    pub fn sender(&self) -> HotkeySender {
        HotkeySender {
            inner: self.sender.clone(),
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> HotkeyReceiver {
        self.sender.subscribe()
    }
}

impl Default for HotkeyBus {
    fn default() -> Self {
        Self::new(16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pressed_event_reaches_subscriber() {
        let bus = HotkeyBus::new(8);
        let mut rx = bus.subscribe();
        bus.sender().send(HotkeyEvent::Pressed);
        let evt = rx.recv().await.unwrap();
        assert_eq!(evt, HotkeyEvent::Pressed);
    }

    #[tokio::test]
    async fn no_panic_when_no_subscribers() {
        // No subscriber — send must be a graceful no-op, not a panic.
        let bus = HotkeyBus::new(4);
        let tx = bus.sender();
        tx.send(HotkeyEvent::Pressed);
        tx.send(HotkeyEvent::Released);
    }
}
