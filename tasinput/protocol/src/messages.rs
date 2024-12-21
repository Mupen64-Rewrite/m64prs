use serde::{Deserialize, Serialize};

use crate::types::PortMask;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMessage {
    pub request_id: u64,
    pub content: HostContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostContent {
    Request(HostRequest),
    Reply(HostReply),
}
impl From<HostRequest> for HostContent {
    fn from(value: HostRequest) -> Self {
        Self::Request(value)
    }
}
impl From<HostReply> for HostContent {
    fn from(value: HostReply) -> Self {
        Self::Reply(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiMessage {
    pub request_id: u64,
    pub content: UiContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiContent {
    Request(UiRequest),
    Reply(UiReply),
}
impl From<UiRequest> for UiContent {
    fn from(value: UiRequest) -> Self {
        Self::Request(value)
    }
}
impl From<UiReply> for UiContent {
    fn from(value: UiReply) -> Self {
        Self::Reply(value)
    }
}

// HOST -> UI REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostRequest {
    Ping,
    Close,
    InitControllers {
        active: PortMask
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UiReply {
    Ack,
}

// UI -> HOST REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiRequest {
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HostReply {
}