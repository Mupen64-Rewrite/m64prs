//! Protocol definitions and communication utilities for TASInput.

pub mod codec;
mod endpoint;
mod messages;
pub mod types;

use std::time::Duration;

pub use endpoint::*;
pub use messages::*;

/// Recommended interval to send ping messages.
pub const PING_INTERVAL: Duration = Duration::from_millis(500);
/// Recommended interval to wait for ping messages.
pub const PING_TIMEOUT: Duration = Duration::from_millis(700);
