use std::{
    error::Error, ffi::c_int, path::{Path, PathBuf}, sync::{Arc, Mutex}
};

use freeze::MovieFreeze;
use m64prs_core::error::M64PError;
use m64prs_sys::Buttons;
use movie::M64Header;
use serde::{Deserialize, Serialize};

pub mod freeze;
pub mod movie;

pub struct VcrState {
    path: PathBuf,
    header: M64Header,
    inputs: Vec<Buttons>,
    index: usize,
    vi_count: usize,
    read_only: bool,
}

impl VcrState {
    pub fn new() -> Self {
        todo!()
    }

    pub fn load_m64<P: AsRef<Path>>(path: P) -> Self {
        todo!()
    }

    pub fn save_m64<P: AsRef<Path>>(&self, path: P) -> Self {
        todo!()
    }

    pub fn filter_inputs(&mut self, _port: c_int, input: Buttons) -> Buttons {
        if self.read_only {
            if self.index < self.inputs.len() {
                let result = self.inputs[self.index];
                self.index += 1;
                result
            } else {
                input
            }
        } else {
            if self.index < self.inputs.len() {
                self.inputs.truncate(self.index);
            }
            self.inputs.push(input);
            input
        }
    }

    pub fn tick_vi(&mut self) {
        self.vi_count += 1;
    }

    pub fn should_exit(&self) -> bool {
        self.read_only && self.index == self.inputs.len()
    }

    pub fn freeze(&self) -> MovieFreeze {
        freeze::v1::MovieFreeze {
            uid: self.header.uid,
            index: self.index.try_into().unwrap(),
            vi_count: self.vi_count.try_into().unwrap(),
            inputs: self.inputs.clone(),
        }.into()
    }

    pub fn load_freeze(&mut self, freeze: MovieFreeze) -> Result<(), M64PError> {
        match freeze {
            MovieFreeze::V1(freeze) => {
                if freeze.uid != self.header.uid {
                    return Err(M64PError::InputInvalid)
                }

                self.index = freeze.index.try_into().unwrap();
                self.vi_count = freeze.vi_count.try_into().unwrap();
                self.inputs = freeze.inputs;

                Ok(())
            },
            #[allow(unreachable_patterns)]
            _ => Err(M64PError::Incompatible)
        }
    }
}
