use m64prs_sys::Buttons;
use serde::{Deserialize, Serialize};

use crate::types::{IpcMessage, IpcPayload, PortMask};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMessage {
    pub id: u64,
    pub payload: IpcPayload<HostRequest, HostReply>,
}

impl IpcMessage for HostMessage {
    type Request = HostRequest;
    type Reply = HostReply;

    fn encode_payload(id: u64, payload: IpcPayload<Self::Request, Self::Reply>) -> Self {
        Self { id, payload }
    }

    fn decode_message(self) -> (u64, IpcPayload<Self::Request, Self::Reply>) {
        let Self { id, payload } = self;
        (id, payload)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiMessage {
    pub id: u64,
    pub payload: IpcPayload<UiRequest, UiReply>,
}

impl IpcMessage for UiMessage {
    type Request = UiRequest;
    type Reply = UiReply;

    fn encode_payload(id: u64, payload: IpcPayload<Self::Request, Self::Reply>) -> Self {
        Self { id, payload }
    }

    fn decode_message(self) -> (u64, IpcPayload<Self::Request, Self::Reply>) {
        let Self { id, payload } = self;
        (id, payload)
    }
}

// HOST -> UI REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostRequest {
    Ping,
    Close,
    InitControllers { active: PortMask },
    SetVisible { visible: bool },
    PollState { controller: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UiReply {
    Ack,
    PolledState { buttons: Buttons },
}

// UI -> HOST REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiRequest {
    Dummy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HostReply {
    Ack,
}
