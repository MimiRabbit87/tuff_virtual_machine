use std::time::{Duration, Instant};

const DURATION_500_000NS: Duration = Duration::from_nanos(500_000);

#[inline]
pub fn sleep(duration: Duration) -> () {
    let start: Instant = Instant::now();

    if duration > DURATION_500_000NS {
        std::thread::sleep(duration - DURATION_500_000NS);
    };

    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}
