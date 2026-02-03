# loose-timer

[![Crates.io](https://img.shields.io/crates/v/loose-timer.svg)](https://crates.io/crates/loose-timer)
[![Docs.rs](https://docs.rs/loose-timer/badge.svg)](https://docs.rs/loose-timer/)
[![Actions Status](https://github.com/frozenlib/loose-timer/workflows/CI/badge.svg)](https://github.com/frozenlib/loose-timer/actions)

ランタイム非依存の時間関連非同期ユーティリティ

## 主な機能

- `sleep`: 非同期版の [`std::thread::sleep`]
- `sleep_until`: 非同期版の [`std::thread::sleep_until`]
- `timeout`, `timeout_at`: [`Future`]をタイムアウト付きにする関数
- `#[timeout]`: テスト関数タイムアウト付きにする属性（同期関数、非同期関数のどちらにも指定可能）
- `#[should_timeout]`: タイムアウトすると成功するテストに付ける属性

[`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
[`std::thread::sleep`]: https://doc.rust-lang.org/std/thread/fn.sleep.html
[`std::thread::sleep_until`]: https://doc.rust-lang.org/std/thread/fn.sleep_until.html

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
