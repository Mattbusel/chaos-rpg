//! Game event system with triggers, handlers, and a priority queue.

use std::collections::HashMap;

/// The category of a game event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Combat,
    Exploration,
    Trade,
    Social,
    Environmental,
    System,
    Custom(String),
}

/// A single game event flowing through the bus.
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub payload: HashMap<String, String>,
    pub timestamp: u64,
    /// Higher number = higher priority (processed first).
    pub priority: u8,
}

/// Condition that determines whether a handler fires for a given event.
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    /// Fires when the event type matches exactly.
    EventTypeMatch(EventType),
    /// Fires when the payload contains the given key–value pair.
    PayloadContains(String, String),
    /// Fires when the event priority is strictly above the threshold.
    PriorityAbove(u8),
    /// Always fires.
    Always,
}

/// A registered handler that reacts to matching events.
#[derive(Debug, Clone)]
pub struct EventHandler {
    pub handler_id: String,
    pub condition: TriggerCondition,
    /// Action tag — simulates a callback by name.
    pub action: String,
}

/// Central event bus with a priority-sorted pending queue.
#[derive(Debug, Default)]
pub struct EventBus {
    handlers: Vec<EventHandler>,
    /// Pending events awaiting processing (sorted high→low priority).
    pending: Vec<GameEvent>,
    /// Fully processed events (history).
    history: Vec<GameEvent>,
}

impl EventBus {
    /// Create a new, empty event bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an event handler.
    pub fn register_handler(&mut self, handler: EventHandler) {
        self.handlers.push(handler);
    }

    /// Check whether an event satisfies a trigger condition.
    pub fn matches(event: &GameEvent, condition: &TriggerCondition) -> bool {
        match condition {
            TriggerCondition::EventTypeMatch(et) => &event.event_type == et,
            TriggerCondition::PayloadContains(k, v) => {
                event.payload.get(k.as_str()).map(|s| s.as_str()) == Some(v.as_str())
            }
            TriggerCondition::PriorityAbove(threshold) => event.priority > *threshold,
            TriggerCondition::Always => true,
        }
    }

    /// Emit an event: push to the pending queue (priority-sorted) and return
    /// the IDs of all handlers that match right now.
    pub fn emit(&mut self, event: GameEvent) -> Vec<String> {
        let triggered: Vec<String> = self
            .handlers
            .iter()
            .filter(|h| Self::matches(&event, &h.condition))
            .map(|h| h.handler_id.clone())
            .collect();

        // Insert into pending sorted descending by priority.
        let pos = self
            .pending
            .partition_point(|e| e.priority >= event.priority);
        self.pending.insert(pos, event);

        triggered
    }

    /// View the pending queue in priority order (high first).
    pub fn pending_events(&self) -> &[GameEvent] {
        &self.pending
    }

    /// Process the highest-priority pending event. Returns the event and the
    /// IDs of handlers that fired for it.
    pub fn process_next(&mut self) -> Option<(GameEvent, Vec<String>)> {
        if self.pending.is_empty() {
            return None;
        }
        let event = self.pending.remove(0);
        let fired: Vec<String> = self
            .handlers
            .iter()
            .filter(|h| Self::matches(&event, &h.condition))
            .map(|h| h.handler_id.clone())
            .collect();
        self.history.push(event.clone());
        Some((event, fired))
    }

    /// Return the last `last_n` processed events (oldest first within the slice).
    pub fn event_history(&self, last_n: usize) -> Vec<&GameEvent> {
        let skip = self.history.len().saturating_sub(last_n);
        self.history.iter().skip(skip).collect()
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn simple_event(id: &str, et: EventType, priority: u8) -> GameEvent {
        GameEvent {
            event_id: id.to_string(),
            event_type: et,
            payload: HashMap::new(),
            timestamp: 0,
            priority,
        }
    }

    fn payload_event(id: &str, key: &str, val: &str) -> GameEvent {
        let mut ev = simple_event(id, EventType::Trade, 5);
        ev.payload.insert(key.to_string(), val.to_string());
        ev
    }

    #[test]
    fn test_emit_returns_triggered_handler_ids() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h1".to_string(),
            condition: TriggerCondition::EventTypeMatch(EventType::Combat),
            action: "fight".to_string(),
        });
        let triggered = bus.emit(simple_event("e1", EventType::Combat, 5));
        assert!(triggered.contains(&"h1".to_string()));
    }

    #[test]
    fn test_emit_no_match() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h1".to_string(),
            condition: TriggerCondition::EventTypeMatch(EventType::Combat),
            action: "fight".to_string(),
        });
        let triggered = bus.emit(simple_event("e1", EventType::Trade, 5));
        assert!(triggered.is_empty());
    }

    #[test]
    fn test_pending_queue_sorted_by_priority() {
        let mut bus = EventBus::new();
        bus.emit(simple_event("low", EventType::System, 1));
        bus.emit(simple_event("high", EventType::System, 10));
        bus.emit(simple_event("mid", EventType::System, 5));

        let pending = bus.pending_events();
        assert_eq!(pending[0].priority, 10);
        assert_eq!(pending[1].priority, 5);
        assert_eq!(pending[2].priority, 1);
    }

    #[test]
    fn test_process_next_returns_highest_priority() {
        let mut bus = EventBus::new();
        bus.emit(simple_event("low", EventType::System, 1));
        bus.emit(simple_event("high", EventType::System, 10));

        let (ev, _) = bus.process_next().unwrap();
        assert_eq!(ev.event_id, "high");
    }

    #[test]
    fn test_process_next_empty() {
        let mut bus = EventBus::new();
        assert!(bus.process_next().is_none());
    }

    #[test]
    fn test_event_history_records_processed_events() {
        let mut bus = EventBus::new();
        bus.emit(simple_event("e1", EventType::Social, 3));
        bus.emit(simple_event("e2", EventType::Trade, 7));
        bus.process_next();
        bus.process_next();
        let hist = bus.event_history(5);
        assert_eq!(hist.len(), 2);
    }

    #[test]
    fn test_event_history_last_n() {
        let mut bus = EventBus::new();
        for i in 0..5_u8 {
            bus.emit(simple_event(&format!("e{}", i), EventType::Exploration, i));
            bus.process_next();
        }
        let hist = bus.event_history(3);
        assert_eq!(hist.len(), 3);
    }

    #[test]
    fn test_payload_contains_condition_match() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h_trade".to_string(),
            condition: TriggerCondition::PayloadContains("item".to_string(), "sword".to_string()),
            action: "log_trade".to_string(),
        });
        let ev = payload_event("e1", "item", "sword");
        let triggered = bus.emit(ev);
        assert!(triggered.contains(&"h_trade".to_string()));
    }

    #[test]
    fn test_payload_contains_condition_no_match() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h_trade".to_string(),
            condition: TriggerCondition::PayloadContains("item".to_string(), "sword".to_string()),
            action: "log_trade".to_string(),
        });
        let ev = payload_event("e1", "item", "shield");
        let triggered = bus.emit(ev);
        assert!(triggered.is_empty());
    }

    #[test]
    fn test_priority_above_condition() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h_urgent".to_string(),
            condition: TriggerCondition::PriorityAbove(5),
            action: "urgent_action".to_string(),
        });
        let low = bus.emit(simple_event("low", EventType::System, 3));
        let high = bus.emit(simple_event("high", EventType::System, 8));
        assert!(low.is_empty());
        assert!(high.contains(&"h_urgent".to_string()));
    }

    #[test]
    fn test_always_condition() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h_always".to_string(),
            condition: TriggerCondition::Always,
            action: "log_all".to_string(),
        });
        let t1 = bus.emit(simple_event("e1", EventType::Combat, 5));
        let t2 = bus.emit(simple_event("e2", EventType::Trade, 1));
        assert!(t1.contains(&"h_always".to_string()));
        assert!(t2.contains(&"h_always".to_string()));
    }

    #[test]
    fn test_multiple_handlers_multiple_triggered() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h1".to_string(),
            condition: TriggerCondition::Always,
            action: "a1".to_string(),
        });
        bus.register_handler(EventHandler {
            handler_id: "h2".to_string(),
            condition: TriggerCondition::EventTypeMatch(EventType::Environmental),
            action: "a2".to_string(),
        });
        let triggered = bus.emit(simple_event("e1", EventType::Environmental, 5));
        assert!(triggered.contains(&"h1".to_string()));
        assert!(triggered.contains(&"h2".to_string()));
    }

    #[test]
    fn test_custom_event_type() {
        let mut bus = EventBus::new();
        bus.register_handler(EventHandler {
            handler_id: "h_custom".to_string(),
            condition: TriggerCondition::EventTypeMatch(EventType::Custom("festival".to_string())),
            action: "celebrate".to_string(),
        });
        let ev = simple_event("e1", EventType::Custom("festival".to_string()), 5);
        let triggered = bus.emit(ev);
        assert!(triggered.contains(&"h_custom".to_string()));
    }
}
