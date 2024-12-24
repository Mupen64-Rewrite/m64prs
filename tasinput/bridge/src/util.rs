use m64prs_sys::Control;

pub(crate) struct ControlsRef {
    controls: *mut Control,
}

impl ControlsRef {
    pub(crate) fn new(controls: *mut Control) -> Self {
        Self { controls }
    }

    pub(crate) unsafe fn get_mut(&mut self, port: usize) -> Option<&mut Control> {
        if port < 4 {
            Some(&mut *(self.controls.offset(port as isize)))
        } else {
            None
        }
    }

    pub(crate) unsafe fn index_mut(&mut self, port: usize) -> &mut Control {
        self.get_mut(port).expect("Port index out of range!")
    }
}

unsafe impl Send for ControlsRef {}
unsafe impl Sync for ControlsRef {}

pub(crate) fn exe_name(name: &str) -> String {
    #[cfg(unix)]
    let result = name.to_owned();
    #[cfg(windows)]
    let result = format!("{}.exe", name);
    result
}
