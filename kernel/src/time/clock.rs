use std::time::Instant;
pub struct MonotonicClock {
    start: Instant,
}
impl MonotonicClock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    pub fn now_ns(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }
}
impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}
pub struct Timer {
    deadline_ns: u64,
}
impl Timer {
    pub fn new(deadline_ns: u64) -> Self {
        Self { deadline_ns }
    }
    pub fn expired(&self, now: u64) -> bool {
        now >= self.deadline_ns
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mono() {
        let c = MonotonicClock::new();
        assert!(c.now_ns() < 1_000_000_000_000);
    }
}
