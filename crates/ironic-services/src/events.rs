//! Typed in-process publish/subscribe events.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};

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

/// A cloneable typed event bus with bounded subscriber queues.
#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<TypeId, Vec<mpsc::Sender<ErasedEvent>>>>>,
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
}
