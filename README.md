# loose-timer

[![Crates.io](https://img.shields.io/crates/v/loose-timer.svg)](https://crates.io/crates/loose-timer)
[![Docs.rs](https://docs.rs/loose-timer/badge.svg)](https://docs.rs/loose-timer/)
[![Actions Status](https://github.com/frozenlib/loose-timer/workflows/CI/badge.svg)](https://github.com/frozenlib/loose-timer/actions)

Runtime-agnostic asynchronous time utilities.

## Key Features

- `sleep`: asynchronous version of [`std::thread::sleep`]
- `sleep_until`: asynchronous version of [`std::thread::sleep_until`]
- `timeout`, `timeout_at`: functions to apply timeouts to [`Future`]
- `#[timeout]`: attribute to add timeouts to test functions (works for both sync and async functions)
- `#[should_timeout]`: attribute for tests that succeed when they time out

[`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
[`std::thread::sleep`]: https://doc.rust-lang.org/std/thread/fn.sleep.html
[`std::thread::sleep_until`]: https://doc.rust-lang.org/std/thread/fn.sleep_until.html

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
