use m64prs_core::tas_callbacks::SaveHandler;


#[derive(Default)]
pub(super) struct TestSaveHandler {
    counter: u64
}

impl SaveHandler for TestSaveHandler {
    const SIGNATURE: u32 = 0x12345678;

    const VERSION: u32 = 0x010000;

    const ALLOC_SIZE: usize = std::mem::size_of::<u64>();

    fn save_extra_data(&mut self, data: &mut [u8]) {
        let value = self.counter;
        log::debug!("saved value {}", value);
        data[0..8].copy_from_slice(&value.to_le_bytes());
        self.counter += 1;
    }

    fn load_extra_data(&mut self, version: u32, data: &[u8]) {
        let value = u64::from_le_bytes(*<&[u8; 8]>::try_from(&data[0..8]).unwrap());
        log::debug!("loaded value {}", value);
    }

    fn get_data_size(&mut self, version: u32) -> usize {
        Self::ALLOC_SIZE
    }
}