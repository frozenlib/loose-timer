use loose_timer::timeout;

#[timeout("1h")]
fn invalid_suffix() {}

fn main() {}
