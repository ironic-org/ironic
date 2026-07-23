//! Typed in-process publish/subscribe events with dead-letter queue support.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock, mpsc};

type ErasedEvent = Arc<dyn Any + Send + Sync>;

/// A typed event subscriber.
///
/// # Errors
///
/// [`recv`](EventSubscription::recv) returns `None` when the bus drops.
///
/// # Panics
///
/// Never panics.
pub struct EventSubscription<E> {
    receiver: mpsc::Receiver<ErasedEvent>,
    marker: std::marker::PhantomData<fn() -> E>,
}

impl<E: Send + Sync + 'static> EventSubscription<E> {
    /// Waits for the next event, or returns `None` when the bus is gone.
    pub async fn recv(&mut self) -> Option<Arc<E>> {
        while let Some(event) = self.receiver.recv().await {
            if let Ok(event) = event.downcast::<E>() {
                return Some(event);
            }
        }
        None
    }
}

/// An undelivered event captured by the dead-letter queue.
#[derive(Clone, Debug)]
pub struct DeadLetter {
    /// The type name of the event.
    pub type_name: &'static str,
    /// The event data.
    pub event: String,
}

/// A cloneable typed event bus with bounded subscriber queues and a dead-letter queue.
#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<TypeId, Vec<mpsc::Sender<ErasedEvent>>>>>,
    dead_letters: Arc<Mutex<Vec<DeadLetter>>>,
}

impl EventBus {
    /// Subscribes to events of type `E` with bounded backpressure.
    pub async fn subscribe<E: Send + Sync + 'static>(
        &self,
        capacity: usize,
    ) -> EventSubscription<E> {
        let (sender, receiver) = mpsc::channel(capacity.max(1));
        self.subscribers
            .write()
            .await
            .entry(TypeId::of::<E>())
            .or_default()
            .push(sender);
        EventSubscription {
            receiver,
            marker: std::marker::PhantomData,
        }
    }

    /// Publishes an event and returns the number of subscribers that accepted it.
    /// Undelivered events are captured in the dead-letter queue.
    pub async fn publish<E: Send + Sync + 'static>(&self, event: E) -> usize {
        let event: ErasedEvent = Arc::new(event);
        let senders = self
            .subscribers
            .read()
            .await
            .get(&TypeId::of::<E>())
            .cloned()
            .unwrap_or_default();
        let mut delivered = 0;
        for sender in senders {
            if sender.send(Arc::clone(&event)).await.is_ok() {
                delivered += 1;
            } else {
                // Store in dead-letter queue on failure
                let entry = DeadLetter {
                    type_name: std::any::type_name::<E>(),
                    event: format!("{event:?}"),
                };
                self.dead_letters.lock().await.push(entry);
            }
        }
        self.subscribers
            .write()
            .await
            .entry(TypeId::of::<E>())
            .or_default()
            .retain(|sender| !sender.is_closed());
        delivered
    }

    /// Returns and clears all undelivered events from the dead-letter queue.
    pub async fn drain_dead_letters(&self) -> Vec<DeadLetter> {
        let mut queue = self.dead_letters.lock().await;
        std::mem::take(&mut *queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestEvent(u32);

    #[tokio::test]
    async fn subscribe_and_recv() {
        let bus = EventBus::default();
        let mut sub = bus.subscribe::<TestEvent>(8).await;
        let n = bus.publish(TestEvent(42)).await;
        assert_eq!(n, 1);
        let ev = sub.recv().await;
        assert_eq!(ev.as_deref(), Some(&TestEvent(42)));
    }

    #[tokio::test]
    async fn multiple_subscribers() {
        let bus = EventBus::default();
        let mut sub1 = bus.subscribe::<TestEvent>(8).await;
        let mut sub2 = bus.subscribe::<TestEvent>(8).await;
        let n = bus.publish(TestEvent(1)).await;
        assert_eq!(n, 2);
        assert_eq!(sub1.recv().await.as_deref(), Some(&TestEvent(1)));
        assert_eq!(sub2.recv().await.as_deref(), Some(&TestEvent(1)));
    }

    #[tokio::test]
    async fn publish_to_no_subscribers() {
        let bus = EventBus::default();
        let n = bus.publish(TestEvent(99)).await;
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn subscribe_different_types() {
        let bus = EventBus::default();
        let mut sub_a = bus.subscribe::<TestEvent>(8).await;
        let mut sub_b = bus.subscribe::<String>(8).await;

        bus.publish(TestEvent(1)).await;
        bus.publish("hello".to_string()).await;

        assert_eq!(sub_a.recv().await.as_deref(), Some(&TestEvent(1)));
        assert_eq!(sub_b.recv().await.as_deref(), Some(&"hello".to_string()));
    }

    #[tokio::test]
    async fn dead_letter_on_dropped_subscriber() {
        let bus = EventBus::default();
        {
            let _sub = bus.subscribe::<TestEvent>(1).await;
        }
        // Give the drop time to propagate
        tokio::task::yield_now().await;
        bus.publish(TestEvent(7)).await;
        let dead = bus.drain_dead_letters().await;
        assert_eq!(dead.len(), 1);
        assert!(dead[0].type_name.contains("TestEvent"));
    }

    #[tokio::test]
    async fn drain_dead_letters_idempotent() {
        let bus = EventBus::default();
        let sub = bus.subscribe::<TestEvent>(1).await;
        drop(sub);
        tokio::task::yield_now().await;
        bus.publish(TestEvent(1)).await;
        assert_eq!(bus.drain_dead_letters().await.len(), 1);
        // second drain returns empty
        assert!(bus.drain_dead_letters().await.is_empty());
    }

    #[tokio::test]
    async fn subscriber_recv_none_when_bus_dropped() {
        let bus = Arc::new(EventBus::default());
        let mut sub = bus.subscribe::<TestEvent>(8).await;
        drop(bus);
        let ev = sub.recv().await;
        assert!(ev.is_none());
    }
}
