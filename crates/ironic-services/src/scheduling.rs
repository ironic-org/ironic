//! Cooperative interval scheduling with deterministic shutdown.

use std::{future::Future, time::Duration};
use tokio::{sync::watch, task::JoinHandle, time::MissedTickBehavior};

/// Handle for one running scheduled task.
pub struct ScheduledTask {
    stop: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl ScheduledTask {
    /// Requests cancellation and waits for the current invocation to finish.
    ///
    /// # Errors
    /// Returns a join error if the task panicked or was externally aborted.
    pub async fn shutdown(self) -> Result<(), tokio::task::JoinError> {
        let _ = self.stop.send(true);
        self.task.await
    }

    /// Immediately aborts the scheduled task.
    pub fn abort(&self) {
        self.task.abort();
    }
}

/// Spawns a fixed interval task. Missed ticks are skipped and the first run follows one period.
#[must_use]
pub fn interval<F, Fut>(period: Duration, task: F) -> ScheduledTask
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (stop, mut stopped) = watch::channel(false);
    let handle = tokio::spawn(async move {
        let mut timer = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = timer.tick() => task().await,
                result = stopped.changed() => {
                    if result.is_err() || *stopped.borrow() { break; }
                }
            }
        }
    });
    ScheduledTask { stop, task: handle }
}
