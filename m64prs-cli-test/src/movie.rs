use std::{fs, path::Path, vec};

use m64prs_core::tas_callbacks::InputHandler;
use m64prs_sys::Buttons;

pub struct MovieInputFilter {
    iter: vec::IntoIter<Buttons>,
    first_poll: bool,
    done: bool,
}

impl MovieInputFilter {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = fs::File::open(path).unwrap();
        let m64_file = m64prs_movie::M64File::read_from(file).unwrap();
        Self {
            iter: m64_file.inputs.into_iter(),
            first_poll: true,
            done: false,
        }
    }
}

impl InputHandler for MovieInputFilter {
    fn filter_inputs(&mut self, port: std::ffi::c_int, input: Buttons) -> Buttons {
        if port != 0 {
            return input;
        }
        if self.done {
            return input;
        }
        if self.first_poll {
            self.first_poll = false;
            return input;
        }
        match self.iter.next() {
            Some(next) => next,
            None => {
                self.done = true;
                input
            }
        }
    }

    fn poll_present(&mut self, port: std::ffi::c_int) -> bool {
        match port {
            0 => true,
            _ => false,
        }
    }
}
