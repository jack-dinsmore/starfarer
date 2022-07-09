use std::time::{Instant, Duration};
use std::thread;

const SAMPLE_COUNT: usize = 5;
const SAMPLE_COUNT_FLOAT: f32 = SAMPLE_COUNT as f32;

pub struct FPSLimiter {
    counter: Instant,
    lower_frame_time: Option<u32>, // unit microseconds
    upper_frame_time: Option<u32>, // unit microseconds
    samples: [u32; SAMPLE_COUNT],
    current_frame: usize,
    delta_frame: u32,
}

impl FPSLimiter {
    pub fn new() -> FPSLimiter {
        FPSLimiter {
            counter: Instant::now(),
            lower_frame_time: None,
            upper_frame_time: None,
            samples: [0; SAMPLE_COUNT],
            current_frame: 0,
            delta_frame: 0,
        }
    }
    pub fn with_limits(lower: Option<u32>, upper: Option<u32>) -> FPSLimiter {
        let lower_frame_time = match lower {
            Some(fps) => Some(1_000_000 / fps),
            None => None,
        };
        let upper_frame_time = match upper {
            Some(fps) => Some(1_000_000 / fps),
            None => None,
        };
        FPSLimiter {
            counter: Instant::now(),
            lower_frame_time,
            upper_frame_time,
            samples: [0; SAMPLE_COUNT],
            current_frame: 0,
            delta_frame: 0,
        }
    }

    /// Call this function in game loop to update its inner status.
    pub fn tick_frame(&mut self) {
        let time_elapsed = self.counter.elapsed();
        self.counter = Instant::now();
        
        self.delta_frame = time_elapsed.subsec_micros();
        self.samples[self.current_frame] = self.delta_frame;
        self.current_frame = (self.current_frame + 1) % SAMPLE_COUNT;
    
        if let Some(t) = self.lower_frame_time {
            let time_left = t as i32 - self.delta_frame as i32;
            if time_left > 0 {
                thread::sleep(Duration::from_micros(time_left as u64));
            }
        }
    
        if let Some(t) = self.upper_frame_time {
            let time_past = self.delta_frame as i32 - t as i32;
            if time_past > 0 {
                // Kill the frame
                self.delta_frame = 0;
            }
        }
    }

    /// Calculate the current FPS.
    pub fn fps(&self) -> f32 {
        let mut sum = 0_u32;
        self.samples.iter().for_each(|val| {
            sum += val;
        });

        1000_000.0_f32 / (sum as f32 / SAMPLE_COUNT_FLOAT)
    }

    /// Return current delta time in seconds.
    pub fn delta_time(&self) -> f32 {
        self.delta_frame as f32 / 1000_000.0_f32 // time in second
    }
}