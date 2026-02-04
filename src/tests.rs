use std::{
    future::Future,
    pin::{Pin, pin},
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};

use assert_call::{Call, CallRecorder, call};
use futures::executor::block_on;
use pretty_assertions::assert_eq;

use super::*;

struct CallWake {
    id: &'static str,
}

impl Wake for CallWake {
    fn wake(self: Arc<Self>) {
        call!("{}", self.id);
    }
}

fn waker(id: &'static str) -> Waker {
    Waker::from(Arc::new(CallWake { id }))
}

fn poll_future(fut: Pin<&mut impl Future<Output = ()>>, waker: &Waker) -> Poll<()> {
    fut.poll(&mut Context::from_waker(waker))
}

#[test]
fn sleep_completes_after_duration() {
    let start = Instant::now();
    block_on(sleep(Duration::from_millis(20)));
    assert!(start.elapsed() >= Duration::from_millis(15));
}

#[test]
fn sleep_until_past_is_ready_immediately() {
    let mut fut = pin!(sleep_until(Instant::now() - Duration::from_millis(1)));
    let waker = Waker::noop();
    assert!(matches!(poll_future(fut.as_mut(), waker), Poll::Ready(())));
}

#[test]
fn updates_waker_when_polled_again() {
    let mut calls = CallRecorder::new();
    let mut fut = pin!(sleep(Duration::from_millis(30)));
    let first = waker("first");
    let second = waker("second");

    assert!(matches!(poll_future(fut.as_mut(), &first), Poll::Pending));
    assert!(matches!(poll_future(fut.as_mut(), &second), Poll::Pending));

    thread::sleep(Duration::from_millis(200));
    calls.verify(["second"]);
}

#[test]
fn earlier_sleep_wakes_first_even_if_registered_later() {
    let mut calls = CallRecorder::new();
    let waker_late = waker("late");
    let waker_early = waker("early");

    let mut late = pin!(sleep(Duration::from_millis(60)));
    let mut early = pin!(sleep(Duration::from_millis(20)));

    assert!(matches!(
        poll_future(late.as_mut(), &waker_late),
        Poll::Pending
    ));
    assert!(matches!(
        poll_future(early.as_mut(), &waker_early),
        Poll::Pending
    ));

    thread::sleep(Duration::from_millis(200));
    calls.verify(["early", "late"]);
}

#[test]
fn drop_removes_sleep_from_queue() {
    let mut calls = CallRecorder::new();
    let mut fut = Box::pin(sleep(Duration::from_millis(30)));
    let wake = waker("dropped");
    assert!(matches!(poll_future(fut.as_mut(), &wake), Poll::Pending));
    drop(fut);

    thread::sleep(Duration::from_millis(120));
    calls.verify(Call::empty());
}

#[test]
fn multiple_sleeps_wake_when_time_is_reached() {
    let mut calls = CallRecorder::new();
    let deadline = Instant::now() + Duration::from_millis(25);
    let mut first_fut = pin!(sleep_until(deadline));
    let mut second_fut = pin!(sleep_until(deadline));
    let first = waker("first");
    let second = waker("second");

    assert!(matches!(
        poll_future(first_fut.as_mut(), &first),
        Poll::Pending
    ));
    assert!(matches!(
        poll_future(second_fut.as_mut(), &second),
        Poll::Pending
    ));

    thread::sleep(Duration::from_millis(200));
    calls.verify(Call::par([["first"], ["second"]]));
}

#[test]
fn promotes_new_wakers_when_earliest_is_removed() {
    let mut calls = CallRecorder::new();
    let waker_first = waker("first");
    let waker_second = waker("second");
    let waker_third = waker("third");

    let mut earliest = Box::pin(sleep(Duration::from_millis(20)));
    let mut second = pin!(sleep(Duration::from_millis(60)));
    let mut third = pin!(sleep(Duration::from_millis(80)));

    assert!(matches!(
        poll_future(earliest.as_mut(), &waker_first),
        Poll::Pending
    ));
    assert!(matches!(
        poll_future(second.as_mut(), &waker_second),
        Poll::Pending
    ));
    assert!(matches!(
        poll_future(third.as_mut(), &waker_third),
        Poll::Pending
    ));

    drop(earliest);

    thread::sleep(Duration::from_millis(200));
    calls.verify(["second", "third"]);
}

#[test]
fn concurrent_sleep_until_does_not_deadlock() {
    let base = Instant::now() + Duration::from_millis(30);
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                block_on(sleep_until(base + Duration::from_millis(i * 5)));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn timeout_sync_completes_before_deadline() {
    let result = timeout_helpers::timeout_sync(|| 42, Duration::from_millis(100));
    assert_eq!(result, Ok(42));
}

#[test]
fn timeout_sync_times_out() {
    let result = timeout_helpers::timeout_sync(
        || {
            thread::sleep(Duration::from_millis(200));
            42
        },
        Duration::from_millis(50),
    );
    assert!(result.is_err());
}
