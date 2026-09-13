#[derive(Debug, Clone, PartialEq, Eq)] pub struct Message { pub from: u32, pub to: u32, pub tag: u32, pub payload: Vec<u8> }
impl Message { pub fn new(from: u32, to: u32, tag: u32, payload: Vec<u8>) -> Self { Self { from, to, tag, payload } } }
