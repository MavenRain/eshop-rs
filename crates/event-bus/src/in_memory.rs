//! In-memory [`EventBus`] implementation for tests.

use std::sync::mpsc::{Receiver, Sender, channel};

use comp_cat_rs::effect::io::Io;

use crate::bus::EventBus;
use crate::error::Error;
use crate::event::IntegrationEvent;

/// Synchronous in-memory bus.  Each [`InMemoryEventBus::publish`] call sends
/// the event to a [`Receiver`] handed to the test at construction time.
///
/// The bus is `Clone` so callers can publish from multiple sites; cloning
/// shares the underlying [`Sender`] but does not duplicate received events.
pub struct InMemoryEventBus<E> {
    tx: Sender<E>,
}

impl<E> InMemoryEventBus<E> {
    /// Construct a fresh bus and the [`Receiver`] used to inspect published
    /// events.
    #[must_use]
    pub fn new() -> (Self, Receiver<E>) {
        let (tx, rx) = channel();
        (Self { tx }, rx)
    }
}

impl<E> Clone for InMemoryEventBus<E> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<E: IntegrationEvent> EventBus<E> for InMemoryEventBus<E> {
    fn publish(&self, event: E) -> Io<Error, ()> {
        let tx = self.tx.clone();
        Io::suspend(move || {
            tx.send(event).map_err(|e| Error::Publish {
                reason: e.to_string(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "event_name")]
    enum TestEvent {
        Foo { value: u32 },
        Bar { tag: String },
    }

    impl IntegrationEvent for TestEvent {
        fn event_name(&self) -> &'static str {
            match self {
                Self::Foo { .. } => "Foo",
                Self::Bar { .. } => "Bar",
            }
        }

        fn all_event_names() -> &'static [&'static str] {
            &["Foo", "Bar"]
        }
    }

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Publish { reason: reason() })
        }
    }

    #[test]
    fn publish_round_trip() -> Result<(), Error> {
        let (bus, rx) = InMemoryEventBus::<TestEvent>::new();
        bus.publish(TestEvent::Foo { value: 7 }).run()?;
        let got = rx.recv().map_err(|e| Error::Publish {
            reason: e.to_string(),
        })?;
        check(got == TestEvent::Foo { value: 7 }, || {
            format!("expected Foo {{ value: 7 }}, got {got:?}")
        })
    }

    #[test]
    fn event_names_dispatch_correctly() -> Result<(), Error> {
        let foo = TestEvent::Foo { value: 1 };
        let bar = TestEvent::Bar {
            tag: "hi".to_string(),
        };
        check(foo.event_name() == "Foo", || {
            format!("expected Foo, got {}", foo.event_name())
        })?;
        check(bar.event_name() == "Bar", || {
            format!("expected Bar, got {}", bar.event_name())
        })
    }

    #[test]
    fn metadata_is_unique_per_call() -> Result<(), Error> {
        let a = IntegrationEventMetadata::fresh();
        let b = IntegrationEventMetadata::fresh();
        check(a.id() != b.id(), || {
            format!("expected unique ids, got {} == {}", a.id(), b.id())
        })
    }

    use crate::event::IntegrationEventMetadata;
}
