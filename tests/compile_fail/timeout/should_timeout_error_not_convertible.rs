use std::time::Duration;

use loose_timer::{should_timeout, sleep};

#[should_timeout("10ms")]
async fn should_timeout_result() -> Result<(), NotConvertible> {
    sleep(Duration::from_millis(20)).await;
    Ok(())
}

#[derive(Debug)]
struct NotConvertible;

fn main() {
    let _ = should_timeout_result();
}
