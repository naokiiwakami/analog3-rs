use embassy_time::{Duration, Instant};

use crate::rng::LocalRng;

#[non_exhaustive]
pub struct BackoffBlinker<'a> {
    rng: &'a LocalRng,
    last_lit: Instant,
    interval_c: u64,
    interval: Duration,
    pub remaining: usize,
    pub index: usize,
    pub turn_on: bool,
}

impl<'a> BackoffBlinker<'a> {
    pub fn new(rng: &'a LocalRng, num_blinks: usize) -> Self {
        let interval_c = 100;
        let interval = Duration::from_millis(rng.random_u64() % interval_c + interval_c / 2);
        Self {
            rng,
            last_lit: Instant::now(),
            interval_c,
            interval,
            remaining: num_blinks,
            index: 0,
            turn_on: false,
        }
    }

    pub fn initial_interval_ms(&mut self, interval: u64) -> &mut Self {
        self.interval_c = interval;
        self
    }

    /// Checks whether it's time to do any blinking action.
    /// The user should check properties `turn_on` and `index` to determine what to do.
    /// If `turn_on` is true, the user should turn on an indicator LED for the `index`.
    pub fn check(&mut self) -> bool {
        if self.remaining > 0 && self.last_lit.elapsed() >= self.interval {
            if self.turn_on {
                self.turn_on = false;
                self.interval = Duration::from_millis(
                    self.rng.random_u64() % self.interval_c + self.interval_c / 2,
                );
                self.remaining -= 1;
                return true;
            } else {
                self.turn_on = true;
                self.index = (self.rng.random_u64() % 2) as usize;
                self.interval_c = self.interval_c * 3 / 2;
                self.interval = Duration::from_millis(30);
                self.last_lit = Instant::now();
                return true;
            }
        }
        false
    }
}
