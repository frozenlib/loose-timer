use std::time::Duration;

use loose_timer::timeout;

#[timeout("10ms")]
fn timeout_result() -> Result<(), NotConvertible> {
    std::thread::sleep(Duration::from_millis(20));
    Ok(())
}

#[derive(Debug)]
struct NotConvertible;

fn main() {
    let _ = timeout_result();
}
