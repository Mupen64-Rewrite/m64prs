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
    Ping,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiReply {
    Pong,
}

// UI -> HOST REQUESTS
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiRequest {}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostReply {}