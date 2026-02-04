//! Runtime-agnostic asynchronous utilities for time-related tasks.

use std::{
    collections::BTreeMap,
    future::{Future, IntoFuture},
    pin::Pin,
    sync::{Condvar, LazyLock, Mutex},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use futures::future::select;
use futures::{future::Either, pin_mut};
use parse_display::Display;
use slabmap::SlabMap;

mod macros;

pub use macros::*;

#[doc(hidden)]
pub mod timeout_helpers;

#[cfg(doctest)]
mod test_readme {
    #[doc = include_str!("../README.md")]
    mod readme {}

    #[doc = include_str!("../README.ja.md")]
    mod readme_ja {}
}

static SLEEP_REGISTRY: LazyLock<SleepRegistry> = LazyLock::new(|| SleepRegistry {
    queue: Mutex::new(SleepQueue::new()),
    condvar: Condvar::new(),
});

struct SleepRegistry {
    queue: Mutex<SleepQueue>,
    condvar: Condvar,
}
impl SleepRegistry {
    fn run_worker(&self) {
        let mut wakes = Vec::new();
        let mut queue = self.queue.lock().unwrap();
        loop {
            let now = Instant::now();
            let q = &mut *queue;
            while let Some(task) = q.tasks.first_entry()
                && task.key().deadline <= now
            {
                wakes.push(q.entries[*task.get()].take().unwrap().waker);
                task.remove();
            }
            if !wakes.is_empty() {
                drop(queue);
                for waker in wakes.drain(..) {
                    waker.wake();
                }
                queue = self.queue.lock().unwrap();
                continue;
            }
            queue = if let Some(task) = queue.tasks.first_key_value() {
                let wait_duration = task.0.deadline.saturating_duration_since(now);
                self.condvar.wait_timeout(queue, wait_duration).unwrap().0
            } else {
                self.condvar.wait(queue).unwrap()
            };
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    deadline: Instant,
    seq: usize,
}

impl Key {
    fn new(deadline: Instant, seq: usize) -> Self {
        Self { deadline, seq }
    }
}

struct Entry {
    waker: Waker,
    key: Key,
}
impl Entry {
    fn set_waker(&mut self, waker: &Waker) {
        if !self.waker.will_wake(waker) {
            self.waker = waker.clone();
        }
    }
}

struct SleepQueue {
    next_seqs: BTreeMap<Instant, usize>,
    tasks: BTreeMap<Key, usize>,
    entries: SlabMap<Option<Entry>>,
    thread_running: bool,
}

impl SleepQueue {
    fn lock() -> std::sync::MutexGuard<'static, SleepQueue> {
        SLEEP_REGISTRY.queue.lock().unwrap()
    }

    fn new() -> Self {
        Self {
            next_seqs: BTreeMap::new(),
            tasks: BTreeMap::new(),
            entries: SlabMap::new(),
            thread_running: false,
        }
    }

    fn insert(&mut self, deadline: Instant, waker: Waker, condvar: &Condvar) -> usize {
        self.ensure_thread_running();
        let next_seq = self.next_seqs.entry(deadline).or_insert(0);
        assert_ne!(
            *next_seq,
            usize::MAX,
            "Too many sleep entries for the same instant"
        );
        let key = Key::new(deadline, *next_seq);
        *next_seq += 1;

        let notify = if let Some(first_task) = self.tasks.first_key_value() {
            key < *first_task.0
        } else {
            true
        };
        let id = self.entries.insert(Some(Entry { waker, key }));
        self.tasks.insert(key, id);
        if notify {
            condvar.notify_one();
        }
        id
    }

    fn ensure_thread_running(&mut self) {
        if self.thread_running {
            return;
        }
        self.thread_running = true;
        std::thread::spawn(move || SLEEP_REGISTRY.run_worker());
    }

    fn poll_or_remove(&mut self, id: usize, cx: &Context) -> Poll<()> {
        if let Some(e) = &mut self.entries[id] {
            e.set_waker(cx.waker());
            Poll::Pending
        } else {
            self.entries.remove(id);
            Poll::Ready(())
        }
    }

    fn remove(&mut self, id: usize) {
        if let Some(e) = self.entries.remove(id).unwrap() {
            self.tasks.remove(&e.key);
        }
    }
}

struct WakeAtTask {
    id: Option<usize>,
}

impl WakeAtTask {
    fn schedule(deadline: Instant, waker: Waker) -> Self {
        Self {
            id: Some(SleepQueue::lock().insert(deadline, waker, &SLEEP_REGISTRY.condvar)),
        }
    }
}

impl Future for WakeAtTask {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(id) = self.id {
            let poll = SleepQueue::lock().poll_or_remove(id, cx);
            if poll.is_ready() {
                self.get_mut().id = None;
            }
            poll
        } else {
            Poll::Ready(())
        }
    }
}
impl Drop for WakeAtTask {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            SleepQueue::lock().remove(id);
        }
    }
}

/// Sleeps for the given duration.
pub async fn sleep(duration: Duration) {
    if duration > Duration::ZERO {
        WakeAtTask::schedule(Instant::now() + duration, Waker::noop().clone()).await
    }
}
/// Sleeps until the given deadline.
pub async fn sleep_until(deadline: Instant) {
    if deadline > Instant::now() {
        WakeAtTask::schedule(deadline, Waker::noop().clone()).await
    }
}

/// Error indicating that an expected timeout did not occur.
#[derive(Debug, Display, PartialEq, Eq)]
#[display("should timeout")]
pub struct ShouldTimeoutError {
    _private: (),
}
impl ShouldTimeoutError {
    fn new() -> Self {
        Self { _private: () }
    }
}

impl std::error::Error for ShouldTimeoutError {}

/// Error indicating that a timeout occurred.
#[derive(Debug, Display, PartialEq, Eq)]
#[display("timeout")]
pub struct TimeoutError {
    _private: (),
}
impl TimeoutError {
    fn new() -> Self {
        Self { _private: () }
    }
}

impl std::error::Error for TimeoutError {}

/// Runs a future with a timeout duration.
pub async fn timeout<T>(
    duration: Duration,
    fut: impl IntoFuture<Output = T>,
) -> Result<T, TimeoutError> {
    let timeout = sleep(duration);
    let fut = fut.into_future();
    pin_mut!(fut);
    pin_mut!(timeout);
    match select(fut, timeout).await {
        Either::Left((value, _)) => Ok(value),
        Either::Right((_, _)) => Err(TimeoutError::new()),
    }
}
/// Runs a future with a timeout deadline.
pub async fn timeout_at<T>(
    deadline: Instant,
    fut: impl IntoFuture<Output = T>,
) -> Result<T, TimeoutError> {
    let timeout = sleep_until(deadline);
    let fut = fut.into_future();
    pin_mut!(fut);
    pin_mut!(timeout);
    match select(fut, timeout).await {
        Either::Left((value, _)) => Ok(value),
        Either::Right((_, _)) => Err(TimeoutError::new()),
    }
}

/// Values that can be converted to a `Duration` for timeouts.
///
/// Strings use the "number + suffix" format (e.g., `"250ms"`, `"1s"`, `"1.5m"`).
/// Only finite values >= 0 are allowed.
///
/// | Suffix | Equivalent to                      |
/// |--------|----------------------------------- |
/// | `ms`   | `Duration::from_millis(n)`         |
/// | `s`    | `Duration::from_secs(n)`           |
/// | `m`    | `Duration::from_secs(n * 60)`      |
///
/// Implemented for `Duration`, `&str`, `String`, and `&String`.
/// Invalid strings cause a panic during conversion.
pub trait IntoTimeoutDuration {
    fn into_timeout_duration(self) -> Duration;
}
impl IntoTimeoutDuration for Duration {
    fn into_timeout_duration(self) -> Duration {
        self
    }
}
impl IntoTimeoutDuration for &str {
    fn into_timeout_duration(self) -> Duration {
        parse_timeout_duration_str(self).unwrap_or_else(|err| panic!("{err}"))
    }
}
impl IntoTimeoutDuration for String {
    fn into_timeout_duration(self) -> Duration {
        parse_timeout_duration_str(self.as_str()).unwrap_or_else(|err| panic!("{err}"))
    }
}
impl IntoTimeoutDuration for &String {
    fn into_timeout_duration(self) -> Duration {
        parse_timeout_duration_str(self.as_str()).unwrap_or_else(|err| panic!("{err}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParseTimeoutDurationError(&'static str);

impl std::fmt::Display for ParseTimeoutDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

fn parse_timeout_duration_str(raw: &str) -> Result<Duration, ParseTimeoutDurationError> {
    if raw.is_empty() {
        return Err(ParseTimeoutDurationError("duration literal is empty"));
    }

    let (number, unit) = if let Some(prefix) = raw.strip_suffix("ms") {
        (prefix, "ms")
    } else if let Some(prefix) = raw.strip_suffix('s') {
        (prefix, "s")
    } else if let Some(prefix) = raw.strip_suffix('m') {
        (prefix, "m")
    } else {
        return Err(ParseTimeoutDurationError("invalid duration literal"));
    };

    if number.is_empty() {
        return Err(ParseTimeoutDurationError("invalid duration literal"));
    }
    let value: f64 = number
        .parse()
        .map_err(|_| ParseTimeoutDurationError("invalid duration number"))?;
    if !value.is_finite() || value < 0.0 {
        return Err(ParseTimeoutDurationError(
            "duration must be non-negative and finite",
        ));
    }

    let secs = match unit {
        "ms" => value / 1000.0,
        "s" => value,
        "m" => value * 60.0,
        _ => unreachable!(),
    };
    Ok(Duration::from_secs_f64(secs))
}

#[cfg(test)]
mod tests;
