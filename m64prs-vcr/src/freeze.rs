use serde::{Deserialize, Serialize};

pub mod v1 {
    use m64prs_sys::Buttons;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MovieFreeze {
        pub uid: u32,
        pub index: u32,
        pub vi_count: u32,
        pub inputs: Vec<Buttons>,
    }

    pub const VERSION_CODE: u32 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MovieFreeze {
    V1(v1::MovieFreeze)
}

impl From<v1::MovieFreeze> for MovieFreeze {
    fn from(value: v1::MovieFreeze) -> Self {
        Self::V1(value)
    }
}