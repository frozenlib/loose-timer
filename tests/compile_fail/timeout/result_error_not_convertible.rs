use std::time::Duration;

use loose_timer::{sleep, timeout};

#[timeout("10ms")]
async fn timeout_result() -> Result<(), NotConvertible> {
    sleep(Duration::from_millis(20)).await;
    Ok(())
}

#[derive(Debug)]
struct NotConvertible;

fn main() {
    let _ = timeout_result();
}
