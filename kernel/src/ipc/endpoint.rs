#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct Handle(pub u32);
#[derive(Debug)] pub struct Endpoint { pub id: u32, pub owner: u32 }
impl Endpoint { pub fn new(id: u32, owner: u32) -> Self { Self { id, owner } } }
