use instant::{Duration, Instant};

pub struct WorldState {
    frames: i32,
    acc_time: Duration,
    time: Instant,
    time_since_last_frame: Duration,
}
