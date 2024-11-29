use std::{ffi::c_int, path::PathBuf, sync::{Arc, Mutex}};

use m64prs_sys::Buttons;
use movie::M64Header;

pub mod movie;
mod schema;

pub struct VcrState {
    path: PathBuf,
    header: M64Header,
    inputs: Vec<Buttons>,
    index: usize,
    read_only: bool,
}



impl VcrState {
    pub fn new() -> Self {
        todo!()
    }

    pub fn load_file(path: PathBuf) -> Self {
        todo!()
    }

    pub fn filter_inputs(&mut self, _port: c_int, input: Buttons) -> Buttons {
        if self.read_only {
            if self.index < self.inputs.len() {
                let result = self.inputs[self.index];
                self.index += 1;
                result
            }
            else {
                input
            }
        }
        else {
            if self.index < self.inputs.len() {
                self.inputs.truncate(self.index);
            }
            self.inputs.push(input);
            input
        }
    }

    pub fn should_exit(&self) -> bool {
        self.read_only && self.index == self.inputs.len()
    }
}
