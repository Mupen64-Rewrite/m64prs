use serde::{Deserialize, Serialize};

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

// HOST -> UI REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostRequest {
    Init,
    Ping,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiReply {
    Init,
    Ping,
}

// UI -> HOST REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiRequest {
    Ping,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostReply {
    Pong,
}