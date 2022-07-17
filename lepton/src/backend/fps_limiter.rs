use std::time::{Instant, Duration};
use std::thread;

const REFRESH_FRAME: usize = 64;

pub struct FPSLimiter {
    counter: Instant,
    lower_frame_time: Option<u32>, // unit microseconds
    upper_frame_time: Option<u32>, // unit microseconds
    current_frame: usize,
    last_micros: u128,
}

impl FPSLimiter {
    pub fn new() -> FPSLimiter {
        FPSLimiter {
            counter: Instant::now(),
            lower_frame_time: None,
            upper_frame_time: None,
            current_frame: 0,
            last_micros: 0,
        }
    }
    pub fn with_limits(lower: Option<u32>, upper: Option<u32>) -> FPSLimiter {
        let lower_frame_time = lower.map(|fps| {1_000_000 / fps} );
        let upper_frame_time = upper.map(|fps| {1_000_000 / fps} );
        FPSLimiter {
            counter: Instant::now(),
            lower_frame_time,
            upper_frame_time,
            current_frame: 0,
            last_micros: 0,
        }
    }

    /// Call this function in game loop to update its inner status.
    pub fn tick_frame(&mut self) -> f32 {
        let mut now_micros = self.counter.elapsed().as_micros();
        let mut delta_frame = now_micros - self.last_micros;

        if let Some(t) = self.lower_frame_time {
            let time_left = t as i128 - delta_frame as i128;
            if time_left > 0 {
                thread::sleep(Duration::from_micros(time_left as u64));
                now_micros = self.counter.elapsed().as_micros();
                delta_frame = now_micros - self.last_micros;
            }
        }
        
        if let Some(t) = self.upper_frame_time {
            let time_past = delta_frame as i128 - t as i128;
            if time_past > 0 {
                // Kill the frame
                delta_frame = 0;
            }
        }

        self.current_frame += 1;
        if self.current_frame == REFRESH_FRAME {
            self.counter = Instant::now();
            self.last_micros = 0;
            self.current_frame = 0;
        } else {
            self.last_micros = now_micros;
        }

        delta_frame as f32 / 1_000_000.0_f32 // time in second
    }
}