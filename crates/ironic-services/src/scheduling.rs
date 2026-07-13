//! Cooperative scheduling with cron expressions, fixed intervals, and deterministic shutdown.
//!
//! - [`interval`] runs a task on a fixed period with [`MissedTickBehavior::Skip`].
//! - [`cron_schedule`] (requires `cron` feature) parses a cron expression and runs the task
//!   on every matching instant.
//! - [`ScheduledTask`] supports cancellation and graceful shutdown.

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

/// Spawns a task driven by a cron expression. The task fires when the system
/// clock matches the schedule. Requires the `cron` feature.
///
/// # Panics
///
/// Panics if the expression cannot be parsed (call [`cron_schedule`] to handle errors).
#[cfg(feature = "cron")]
#[must_use]
pub fn cron<F, Fut>(expression: &str, task: F) -> ScheduledTask
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let schedule = expression
        .parse::<::cron::Schedule>()
        .expect("invalid cron expression");
    let (stop, mut stopped) = watch::channel(false);
    let handle = tokio::spawn(async move {
        loop {
            let next = schedule.upcoming(chrono::Utc).next();
            let Some(next_instant) = next else {
                break;
            };
            let delay = (next_instant - chrono::Utc::now())
                .to_std()
                .unwrap_or(Duration::ZERO);
            tokio::select! {
                () = tokio::time::sleep(delay) => task().await,
                result = stopped.changed() => {
                    if result.is_err() || *stopped.borrow() { break; }
                }
            }
        }
    });
    ScheduledTask { stop, task: handle }
}

/// Parses a cron expression and spawns a scheduled task.
///
/// # Errors
///
/// Returns a human-readable error when `expression` is not a valid cron string.
#[cfg(feature = "cron")]
pub fn cron_schedule<F, Fut>(expression: &str, task: F) -> Result<ScheduledTask, String>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let schedule = expression
        .parse::<::cron::Schedule>()
        .map_err(|error| format!("invalid cron expression `{expression}`: {error}"))?;
    let (stop, mut stopped) = watch::channel(false);
    let handle = tokio::spawn(async move {
        loop {
            let next = schedule.upcoming(chrono::Utc).next();
            let Some(next_instant) = next else {
                break;
            };
            let delay = (next_instant - chrono::Utc::now())
                .to_std()
                .unwrap_or(Duration::ZERO);
            tokio::select! {
                () = tokio::time::sleep(delay) => task().await,
                result = stopped.changed() => {
                    if result.is_err() || *stopped.borrow() { break; }
                }
            }
        }
    });
    Ok(ScheduledTask { stop, task: handle })
}
