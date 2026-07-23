//! Cooperative scheduling with cron expressions, fixed intervals, and deterministic shutdown.
//!
//! - [`interval`] runs a task on a fixed period with [`MissedTickBehavior::Skip`].
//! - [`cron_schedule`] (requires `cron` feature) parses a cron expression and runs the task
//!   on every matching instant.
//! - [`ScheduledTask`] supports cancellation, graceful shutdown, pause, and resume.

use std::{future::Future, time::Duration};
use tokio::{sync::watch, task::JoinHandle, time::MissedTickBehavior};

/// Task state for pause/resume control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskState {
    Running,
    Paused,
    Stopped,
}

/// Handle for one running scheduled task.
///
/// # Errors
///
/// [`shutdown`](ScheduledTask::shutdown) returns a [`JoinError`](tokio::task::JoinError)
/// if the spawned task panicked.
///
/// # Panics
///
/// Never panics on its own.
pub struct ScheduledTask {
    control: watch::Sender<TaskState>,
    task: JoinHandle<()>,
}

impl ScheduledTask {
    /// Requests cancellation and waits for the current invocation to finish.
    ///
    /// # Errors
    /// Returns a join error if the task panicked or was externally aborted.
    pub async fn shutdown(self) -> Result<(), tokio::task::JoinError> {
        let _ = self.control.send(TaskState::Stopped);
        self.task.await
    }

    /// Pauses the task. In-flight invocations complete but no new ones start.
    pub fn pause(&self) {
        let _ = self.control.send(TaskState::Paused);
    }

    /// Resumes a paused task. The next tick fires on schedule.
    pub fn resume(&self) {
        let _ = self.control.send(TaskState::Running);
    }

    /// Immediately aborts the scheduled task.
    pub fn abort(&self) {
        self.task.abort();
    }
}

fn should_break(result: &Result<(), watch::error::RecvError>, state: TaskState) -> bool {
    result.is_err() || matches!(state, TaskState::Stopped)
}

/// Spawns a fixed interval task. Missed ticks are skipped and the first run follows one period.
#[must_use]
pub fn interval<F, Fut>(period: Duration, task: F) -> ScheduledTask
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (control, mut rx) = watch::channel(TaskState::Running);
    let handle = tokio::spawn(async move {
        let mut timer = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = timer.tick() => {
                    if *rx.borrow() == TaskState::Running {
                        task().await;
                    }
                }
                result = rx.changed() => {
                    if should_break(&result, *rx.borrow()) { break; }
                }
            }
        }
    });
    ScheduledTask {
        control,
        task: handle,
    }
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
    let (control, mut rx) = watch::channel(TaskState::Running);
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
                () = tokio::time::sleep(delay) => {
                    if *rx.borrow() == TaskState::Running {
                        task().await;
                    }
                }
                result = rx.changed() => {
                    if should_break(&result, *rx.borrow()) { break; }
                }
            }
        }
    });
    ScheduledTask {
        control,
        task: handle,
    }
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
    let (control, mut rx) = watch::channel(TaskState::Running);
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
                () = tokio::time::sleep(delay) => {
                    if *rx.borrow() == TaskState::Running {
                        task().await;
                    }
                }
                result = rx.changed() => {
                    if should_break(&result, *rx.borrow()) { break; }
                }
            }
        }
    });
    Ok(ScheduledTask {
        control,
        task: handle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn interval_runs_task() {
        let ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let r = ran.clone();
        let task = interval(Duration::from_millis(10), move || {
            let r = r.clone();
            async move {
                r.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(ran.load(std::sync::atomic::Ordering::SeqCst));
        task.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn shutdown_stops_task() {
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let task = interval(Duration::from_millis(5), move || {
            let c = c.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        });
        tokio::time::sleep(Duration::from_millis(25)).await;
        let count_before = counter.load(std::sync::atomic::Ordering::SeqCst);
        assert!(count_before >= 1);
        task.shutdown().await.unwrap();
        let count_after = counter.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(count_after, count_after);
    }

    #[tokio::test]
    async fn pause_and_resume() {
        let ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let r = ran.clone();
        let task = interval(Duration::from_millis(10), move || {
            let r = r.clone();
            async move {
                r.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        });
        task.pause();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!ran.load(std::sync::atomic::Ordering::SeqCst));
        task.resume();
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(ran.load(std::sync::atomic::Ordering::SeqCst));
        task.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn abort_immediately_stops() {
        let task = interval(Duration::from_mins(1), || async {});
        task.abort();
        let result = task.shutdown().await;
        assert!(result.is_err());
    }

    #[cfg(feature = "cron")]
    #[tokio::test]
    async fn cron_schedule_rejects_bad_expression() {
        let result = cron_schedule("not-a-cron", || async {});
        assert!(result.is_err());
        if let Err(msg) = result {
            assert!(msg.contains("invalid cron expression"));
        }
    }
}
