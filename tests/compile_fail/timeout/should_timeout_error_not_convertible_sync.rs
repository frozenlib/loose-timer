use std::time::Duration;

use loose_timer::should_timeout;

#[should_timeout("10ms")]
fn should_timeout_result() -> Result<(), NotConvertible> {
    std::thread::sleep(Duration::from_millis(20));
    Ok(())
}

#[derive(Debug)]
struct NotConvertible;

fn main() {
    let _ = should_timeout_result();
}
