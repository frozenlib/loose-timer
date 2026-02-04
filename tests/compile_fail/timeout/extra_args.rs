use loose_timer::timeout;

#[timeout(1, 2)]
fn extra_args() {}

fn main() {}
