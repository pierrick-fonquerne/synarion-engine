//! Time utilities.

pub use std::time::{Duration, Instant};

/// A timer for measuring elapsed time.
#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Creates a new timer.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Returns the elapsed time since the timer was created.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Resets the timer.
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Returns the elapsed time in seconds.
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed().as_secs_f32()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
