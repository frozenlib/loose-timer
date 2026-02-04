use loose_timer::timeout;

#[timeout("-1s")]
fn negative_value() {}

fn main() {}
