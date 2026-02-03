use std::{
    future::Future,
    time::{Duration, Instant},
};

use futures::{future::Either, future::select, pin_mut};

use super::{ShouldTimeoutError, TimeoutError, sleep, sleep_until};

#[doc(hidden)]
pub async fn with_should_timeout_async(
    fut: impl Future<Output = ()>,
    duration: Duration,
) -> Result<(), ShouldTimeoutError> {
    let timeout = sleep(duration);
    pin_mut!(fut);
    pin_mut!(timeout);
    match select(fut, timeout).await {
        Either::Left(((), _)) => Err(ShouldTimeoutError::new()),
        Either::Right(((), _fut)) => Ok(()),
    }
}

#[doc(hidden)]
pub async fn with_should_timeout_until_async(
    fut: impl Future<Output = ()>,
    instant: Instant,
) -> Result<(), ShouldTimeoutError> {
    let timeout = sleep_until(instant);
    pin_mut!(fut);
    pin_mut!(timeout);
    match select(fut, timeout).await {
        Either::Left(((), _)) => Err(ShouldTimeoutError::new()),
        Either::Right(((), _fut)) => Ok(()),
    }
}

#[doc(hidden)]
pub fn with_should_timeout(
    f: impl FnOnce() + Send + 'static,
    duration: Duration,
) -> Result<(), ShouldTimeoutError> {
    with_should_timeout_until(f, Instant::now() + duration)
}

#[doc(hidden)]
pub fn with_should_timeout_until(
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

#[doc(hidden)]
pub fn timeout_sync<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
    duration: Duration,
) -> Result<T, TimeoutError> {
    timeout_at_sync(f, Instant::now() + duration)
}

#[doc(hidden)]
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
