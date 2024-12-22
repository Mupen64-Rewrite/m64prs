use bitflags::bitflags;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PortMask: u8 {
        const PORT1 = 1 << 0;
        const PORT2 = 1 << 1;
        const PORT3 = 1 << 2;
        const PORT4 = 1 << 3;
    }
}

impl PortMask {
    pub fn is_port_active(self, port: u8) -> bool {
        if port >= 4 {
            panic!("Invalid port!");
        }
        self.contains(Self::from_bits_retain(1u8 << port))
    }
}

/// An IPC message type, associated with some endpoint.
pub trait IpcMessage: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    type Request: Serialize + DeserializeOwned + Clone + Send + Sync + 'static;
    type Reply: Serialize + DeserializeOwned + Clone + Send + Sync + 'static;

    /// Encodes an arbitrary payload to an IPC message.
    fn encode_payload(id: u64, payload: IpcPayload<Self::Request, Self::Reply>) -> Self;
    /// Encodes a request to an IPC message.
    fn encode_request(id: u64, request: Self::Request) -> Self {
        Self::encode_payload(id, IpcPayload::Request(request))
    }
    /// Encodes a reply to an IPC message.
    fn encode_reply(id: u64, reply: Self::Reply) -> Self {
        Self::encode_payload(id, IpcPayload::Reply(reply))
    }
    /// Decodes an IPC message.
    fn decode_message(self) -> (u64, IpcPayload<Self::Request, Self::Reply>);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcPayload<Request, Reply>
{
    /// A request.
    Request(Request),
    /// A reply.
    Reply(Reply),
}