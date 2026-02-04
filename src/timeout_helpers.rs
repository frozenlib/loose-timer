use std::{
    future::{Future, poll_fn},
    pin::pin,
    task::Poll,
    time::{Duration, Instant},
};

use super::{ShouldTimeoutError, TimeoutError, sleep, sleep_until};

pub async fn should_timeout(
    fut: impl Future<Output = ()>,
    duration: Duration,
) -> Result<(), ShouldTimeoutError> {
    let mut fut = pin!(fut);
    let mut timeout = pin!(sleep(duration));
    poll_fn(|cx| {
        if fut.as_mut().poll(cx).is_ready() {
            Poll::Ready(Err(ShouldTimeoutError::new()))
        } else if timeout.as_mut().poll(cx).is_ready() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    })
    .await
}

pub async fn should_timeout_until(
    fut: impl Future<Output = ()>,
    instant: Instant,
) -> Result<(), ShouldTimeoutError> {
    let mut fut = pin!(fut);
    let mut timeout = pin!(sleep_until(instant));
    poll_fn(|cx| {
        if fut.as_mut().poll(cx).is_ready() {
            Poll::Ready(Err(ShouldTimeoutError::new()))
        } else if timeout.as_mut().poll(cx).is_ready() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    })
    .await
}

pub fn should_timeout_sync(
    f: impl FnOnce() + Send + 'static,
    duration: Duration,
) -> Result<(), ShouldTimeoutError> {
    should_timeout_until_sync(f, Instant::now() + duration)
}

pub fn should_timeout_until_sync(
    f: impl FnOnce() + Send + 'static,
    instant: Instant,
) -> Result<(), ShouldTimeoutError> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        f();
        let _ = tx.send(());
    });
    let timeout = instant.saturating_duration_since(Instant::now());
    match rx.recv_timeout(timeout) {
        Ok(()) => Err(ShouldTimeoutError::new()),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(()),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(ShouldTimeoutError::new()),
    }
}

pub fn timeout_sync<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
    duration: Duration,
) -> Result<T, TimeoutError> {
    timeout_at_sync(f, Instant::now() + duration)
}

pub fn timeout_at_sync<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
    instant: Instant,
) -> Result<T, TimeoutError> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    let timeout = instant.saturating_duration_since(Instant::now());
    rx.recv_timeout(timeout).map_err(|_| TimeoutError::new())
}
