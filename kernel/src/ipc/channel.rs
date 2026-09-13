use super::message::Message;
use std::collections::VecDeque;
#[derive(Debug)] pub enum ChannelError { Full, Empty, Closed }
pub struct Channel { q: VecDeque<Message>, cap: usize, closed: bool }
impl Channel {
    pub fn new(cap: usize) -> Self { Self { q: VecDeque::new(), cap, closed: false } }
    pub fn send(&mut self, m: Message) -> Result<(), ChannelError> {
        if self.closed { return Err(ChannelError::Closed); }
        if self.q.len() >= self.cap { return Err(ChannelError::Full); }
        self.q.push_back(m); Ok(())
    }
    pub fn recv(&mut self) -> Result<Message, ChannelError> { self.q.pop_front().ok_or(ChannelError::Empty) }
    pub fn close(&mut self) { self.closed = true; }
}
#[cfg(test)] mod tests { use super::*; #[test] fn chan() { let mut c = Channel::new(1); c.send(Message::new(1,2,0,vec![1])).unwrap(); assert!(c.send(Message::new(1,2,0,vec![2])).is_err()); assert_eq!(c.recv().unwrap().payload, vec![1]); } }
