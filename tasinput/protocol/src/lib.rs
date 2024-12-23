//! Protocol definitions and communication utilities for TASInput.

pub mod codec;
mod messages;
pub mod types;
mod endpoint;


use std::time::Duration;

pub use messages::*;
pub use endpoint::*;

/// Recommended interval to send ping messages.
pub const PING_INTERVAL: Duration = Duration::from_secs(5);
/// Recommended interval to wait for ping messages.
pub const PING_TIMEOUT: Duration = Duration::from_secs(10);