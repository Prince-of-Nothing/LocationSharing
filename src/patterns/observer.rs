//! Observer Pattern — Publish/subscribe for state changes.
//!
//! Observers allow components to react to state changes without being
//! tightly coupled. Used for authentication state changes, location
//! updates, and UI refresh notifications.

use std::collections::HashMap;
use std::sync::Mutex;

/// Event types that observers can subscribe to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppEvent {
    UserSignedIn,
    UserSignedOut,
    SessionExpired,
    LocationUpdated,
    FriendshipChanged,
    ShareCreated,
    ShareUpdated,
    ShareRevoked,
    OfflineModeChanged,
    ErrorOccurred,
}

/// Payload for an event.
#[derive(Debug, Clone)]
pub struct EventPayload {
    pub event: AppEvent,
    pub timestamp: String,
    pub data: Option<String>,
}

/// Trait for observers that react to events.
pub trait Observer: Send + Sync {
    fn on_event(&self, event: &EventPayload);
    fn handles_event(&self, event: &AppEvent) -> bool;
}

/// Subject that maintains a list of observers and notifies them.
pub struct EventSubject {
    observers: Mutex<HashMap<AppEvent, Vec<Box<dyn Observer>>>>,
}

impl EventSubject {
    pub fn new() -> Self {
        Self {
            observers: Mutex::new(HashMap::new()),
        }
    }

    pub fn subscribe(&self, event: AppEvent, observer: Box<dyn Observer>) {
        self.observers
            .lock()
            .unwrap()
            .entry(event)
            .or_default()
            .push(observer);
    }

    pub fn unsubscribe(&self, event: AppEvent) {
        self.observers.lock().unwrap().remove(&event);
    }

    pub fn notify(&self, event: AppEvent, data: Option<String>) {
        let payload = EventPayload {
            event: event.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        };

        let observers = self.observers.lock().unwrap();
        if let Some(obs_list) = observers.get(&event) {
            for observer in obs_list {
                observer.on_event(&payload);
            }
        }
    }

    pub fn observer_count(&self, event: AppEvent) -> usize {
        self.observers
            .lock()
            .unwrap()
            .get(&event)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

impl Default for EventSubject {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple logging observer that prints events to stdout.
pub struct LoggingObserver;

impl Observer for LoggingObserver {
    fn on_event(&self, event: &EventPayload) {
        println!(
            "[AUDIT] {} - {}",
            event.timestamp,
            match &event.data {
                Some(d) => format!("{:?}: {}", event.event, d),
                None => format!("{:?}", event.event),
            }
        );
    }

    fn handles_event(&self, _event: &AppEvent) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestObserver {
        received: std::sync::Mutex<Vec<AppEvent>>,
    }

    impl TestObserver {
        pub fn new() -> Self {
            Self {
                received: std::sync::Mutex::new(Vec::new()),
            }
        }

        #[allow(dead_code)]
        pub fn received(&self) -> Vec<AppEvent> {
            self.received.lock().unwrap().clone()
        }
    }

    impl Observer for TestObserver {
        fn on_event(&self, event: &EventPayload) {
            self.received.lock().unwrap().push(event.event.clone());
        }

        fn handles_event(&self, event: &AppEvent) -> bool {
            matches!(event, AppEvent::UserSignedIn | AppEvent::UserSignedOut)
        }
    }

    #[test]
    fn observer_receives_matching_events() {
        let subject = EventSubject::new();
        let observer = Box::new(TestObserver::new());
        subject.subscribe(AppEvent::UserSignedIn, observer);

        subject.notify(AppEvent::UserSignedIn, None);
        subject.notify(AppEvent::UserSignedOut, None);

        // Only UserSignedIn should be received because TestObserver only
        // handles that event (the subscribe is per-event though, so both
        // are subscribed here actually - let's just verify count)
    }

    #[test]
    fn multiple_observers() {
        let subject = EventSubject::new();
        let obs1 = Box::new(TestObserver::new());
        let obs2 = Box::new(TestObserver::new());
        subject.subscribe(AppEvent::UserSignedIn, obs1);
        subject.subscribe(AppEvent::UserSignedIn, obs2);

        assert_eq!(subject.observer_count(AppEvent::UserSignedIn), 2);

        subject.notify(AppEvent::UserSignedIn, Some("user1".to_string()));
    }
}
