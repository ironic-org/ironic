//! Typed in-process publish/subscribe events with dead-letter queue support.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock, mpsc};

type ErasedEvent = Arc<dyn Any + Send + Sync>;

/// A typed event subscriber.
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
