/// Adds a timeout to a function.
///
/// ```rust,ignore
/// #[timeout(<duration>)]
/// async fn func() { }
/// ```
/// - duration: Specify the timeout duration using a [`Duration`](std::time::Duration) value
///   like `Duration::from_secs(10)`, or a string like `"10s"` or `"1.5m"`. (See
///   [`IntoTimeoutDuration`](crate::IntoTimeoutDuration) for details.)
///
/// This attribute can be applied to both synchronous and asynchronous functions.
///
/// # Behavior
///
/// - If the function returns `Result<T, E>`, a timeout returns `Err` with
///   [`TimeoutError`](crate::TimeoutError) converted via `Into::into`.
/// - If the function does not return `Result`, a timeout causes a panic.
///
/// # Examples
///
/// ```no_run
/// use loose_timer::{timeout, TimeoutError};
/// use std::time::Duration;
///
/// #[test]
/// #[timeout("10s")]
/// fn timeout_in_tests() -> Result<(), TimeoutError> {
///     Ok(())
/// }
///
/// #[test]
/// #[timeout(Duration::from_millis(1))]
/// fn timeout_sync_no_result() {}
///
/// #[test]
/// #[timeout("1.5m")]
/// async fn timeout_async_result() -> Result<(), TimeoutError> {
///     Ok(())
/// }
///
/// #[test]
/// #[timeout(Duration::from_millis(1))]
/// async fn timeout_async_no_result() {}
/// ```
pub use loose_timer_macros::timeout;

/// Ensures a function times out; if it doesn't, returns an error or panics.
///
/// ```rust,ignore
/// #[should_timeout(<duration>)]
/// async fn func() { }
/// ```
///
/// - duration: Specify the timeout duration using a [`Duration`](std::time::Duration) value
///   like `Duration::from_secs(10)`, or a string like `"10s"` or `"1.5m"`. (See
///   [`IntoTimeoutDuration`](crate::IntoTimeoutDuration) for details.)
///
/// This attribute can be applied to both synchronous and asynchronous functions.
///
/// # Behavior
///
/// - Only functions returning `()` or `Result<(), E>` are supported.
/// - If the function returns `Result<(), E>`, completing before the timeout returns `Err` with
///   [`ShouldTimeoutError`](crate::ShouldTimeoutError)
///   converted via `Into::into`.
/// - If the function returns `()`, completing before the timeout causes a panic.
/// - If the timeout elapses, control returns immediately with `()` or `Ok(())`.
///
/// # Examples
///
/// ```no_run
/// use loose_timer::{should_timeout, ShouldTimeoutError};
/// use std::time::Duration;
///
/// #[test]
/// #[should_timeout("10s")]
/// fn should_timeout_in_tests() -> Result<(), ShouldTimeoutError> {
///     std::thread::sleep(std::time::Duration::from_millis(10));
///     Ok(())
/// }
///
/// #[test]
/// #[should_timeout(Duration::from_millis(1))]
/// fn should_timeout_sync_no_result() {
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// }
///
/// #[test]
/// #[should_timeout("1.5m")]
/// async fn should_timeout_async_result() -> Result<(), ShouldTimeoutError> {
///     std::thread::sleep(std::time::Duration::from_millis(10));
///     Ok(())
/// }
///
/// #[test]
/// #[should_timeout(Duration::from_millis(1))]
/// async fn should_timeout_async_no_result() {
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// }
/// ```
pub use loose_timer_macros::should_timeout;
