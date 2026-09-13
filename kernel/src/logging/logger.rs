#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] pub enum Level { Trace, Debug, Info, Warn, Error }
pub struct Logger { level: Level }
impl Logger {
    pub fn new(level: Level) -> Self { Self { level } }
    pub fn log(&self, l: Level, msg: &str) { if l >= self.level { println!("[{:?}] {}", l, msg); } }
}
#[cfg(test)] mod tests { use super::*; #[test] fn log() { let l = Logger::new(Level::Info); l.log(Level::Info, "hi"); } }
