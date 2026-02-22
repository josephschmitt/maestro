use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaestroEvent {
    pub event_type: String,
    pub scope: Option<String>,
    pub payload: serde_json::Value,
}

pub struct EventBus {
    sender: broadcast::Sender<MaestroEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn emit(&self, event: MaestroEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MaestroEvent> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn emit_without_subscribers_does_not_panic() {
        let bus = EventBus::new();
        bus.emit(MaestroEvent {
            event_type: "test".into(),
            scope: None,
            payload: serde_json::Value::Null,
        });
    }

    #[tokio::test]
    async fn subscriber_receives_emitted_event() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.emit(MaestroEvent {
            event_type: "cards-changed".into(),
            scope: Some("project-123".into()),
            payload: serde_json::json!({"card_id": "abc"}),
        });

        let event = rx.recv().await.expect("should receive event");
        assert_eq!(event.event_type, "cards-changed");
        assert_eq!(event.scope.as_deref(), Some("project-123"));
        assert_eq!(event.payload["card_id"], "abc");
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_event() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.emit(MaestroEvent {
            event_type: "agent-output".into(),
            scope: None,
            payload: serde_json::json!({"line": "hello"}),
        });

        let e1 = rx1.recv().await.expect("rx1 should receive");
        let e2 = rx2.recv().await.expect("rx2 should receive");
        assert_eq!(e1.event_type, "agent-output");
        assert_eq!(e2.event_type, "agent-output");
    }
}
