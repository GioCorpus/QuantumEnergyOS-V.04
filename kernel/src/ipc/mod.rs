pub mod channel; pub mod message; pub mod endpoint;
pub use channel::{Channel, ChannelError};
pub use message::Message;
pub use endpoint::{Endpoint, Handle};
