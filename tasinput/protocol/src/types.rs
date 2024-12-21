use bitflags::bitflags;
use serde::{Deserialize, Serialize};

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
