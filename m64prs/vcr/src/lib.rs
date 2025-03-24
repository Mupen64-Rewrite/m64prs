use std::{
    error::Error,
    ffi::{c_int, OsStr},
    fs, io,
    path::PathBuf,
};

use freeze::MovieFreeze;
use m64prs_core::{error::M64PError, save::SavestateFormat, Core};
use m64prs_sys::{Buttons, EmuState};
use movie::{M64File, M64Header, StartType};

pub mod freeze;
pub mod movie;

/// Struct implementing movie recording state.
/// Designed to play well with core hooks.
#[derive(Debug)]
pub struct VcrState {
    path: PathBuf,
    header: M64Header,
    inputs: Vec<Buttons>,
    index: u32,
    vi_count: u32,
    read_only: bool,
    first_poll: bool,
}

impl VcrState {
    /// Initialize VCR for a new recording.
    pub fn new<P: Into<PathBuf>>(path: P, header: M64Header, read_only: bool) -> Self {
        let path = path.into();
        Self {
            path,
            header,
            inputs: Vec::new(),
            index: 0,
            vi_count: 0,
            read_only,
            first_poll: false,
        }
    }

    /// Initialize VCR with an existing .m64 file.
    pub fn with_m64<P: Into<PathBuf>>(path: P, file: M64File, read_only: bool) -> Self {
        let path = path.into();
        let M64File { header, inputs } = file;
        Self {
            path,
            header,
            inputs,
            index: 0,
            vi_count: 0,
            read_only,
            first_poll: false,
        }
    }

    /// Export the current VCR state to an .m64 file.
    pub fn export(&self) -> (PathBuf, M64File) {
        let mut header = self.header.clone();

        header.length_samples = self.inputs.len().try_into().unwrap();
        header.length_vis = self.vi_count;

        let inputs = self.inputs.clone();

        (self.path.clone(), M64File { header, inputs })
    }

    /// Resets all counters to frame 0 and sets up the core to restart playback.
    /// If `new` is set, creates any necessary auxilliary files instead of loading them.
    pub async fn reset(&mut self, core: &Core, new: bool) -> Result<(), Box<dyn Error>> {
        self.vi_count = 0;
        self.index = 0;

        match self.header.start_flags {
            StartType::FROM_RESET => {
                core.reset(true)?;
                self.first_poll = true;
            }
            StartType::FROM_SNAPSHOT => match new {
                false => {
                    self.first_poll = true;
                    let file_stem = self
                        .path
                        .file_name()
                        .and_then(OsStr::to_str)
                        .and_then(|name_str| name_str.find('.').map(|pos| &name_str[..pos]))
                        .ok_or(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            ".m64 filenames should not start with `.`",
                        ))?;

                    let st_path = fs::read_dir(self.path.parent().unwrap())?
                        .filter_map(|entry| {
                            let entry = entry.unwrap();
                            if !entry.file_type().is_ok_and(|ty| ty.is_file()) {
                                return None;
                            }

                            if !(Some(&*entry.file_name())
                                .and_then(OsStr::to_str)
                                .and_then(|name_str| name_str.find('.').map(|pos| &name_str[..pos]))
                                .is_some_and(|stem| stem == file_stem))
                            {
                                return None;
                            }

                            Some(entry.path())
                        })
                        .next()
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                "No .st file found for .m64 movie",
                            )
                        })?;

                    core.load_file(st_path).await?;
                }
                true => {
                    self.first_poll = true;
                    let st_path = self.path.with_extension("st");
                    core.save_file(st_path, SavestateFormat::Mupen64Plus)
                        .await?;
                }
            },
            StartType::FROM_EEPROM => {
                unimplemented!()
            }
            _ => panic!("invalid start flags"),
        }
        log::info!("VCR restart");

        Ok(())
    }

    /// Implementation of [`InputHandler::filter_inputs`][m64prs_core::tas_callbacks::InputHandler::filter_inputs].  
    /// This method will either play back inputs (read/write mode), or overwrite inputs, depending on the read-only mode.
    /// # Return value
    /// Two things:
    /// - `Buttons`: the filtered input value
    /// - `bool`: if true, the VCR state has run out of frames.
    pub fn filter_inputs(&mut self, port: c_int, input: Buttons) -> (Buttons, bool) {
        // don't overwrite inputs we don't care about
        if !self.header.controller_flags.port_present(port) {
            return (input, false);
        }

        // this works for singleplayer .m64s, not sure about multiplayer
        if self.first_poll {
            self.first_poll = false;
            return (input, false);
        }

        let index_usize: usize = self.index.try_into().unwrap();
        if self.read_only {
            if index_usize < self.inputs.len() {
                let result = self.inputs[index_usize];
                self.index += 1;
                (result, false)
            } else {
                (input, true)
            }
        } else {
            if index_usize < self.inputs.len() {
                self.inputs.truncate(index_usize);
            }
            // TODO: account for the (2**32 - 1)th frame being the last
            self.inputs.push(input);
            self.index += 1;
            (input, false)
        }
    }

    /// Implementation of [`InputHandler::poll_present`][m64prs_core::tas_callbacks::InputHandler::poll_present].  
    /// This method will return true for any port where input is being recorded.
    pub fn poll_present(&self, port: c_int) -> bool {
        self.header.controller_flags.port_present(port)
    }

    /// Implementation of [`m64prs_core::tas_callbacks::FrameHandler`]. This method
    /// simply increments the VI count.
    pub fn tick_vi(&mut self) {
        if self.read_only {
            if usize::try_from(self.index).unwrap() < self.inputs.len() {
                self.vi_count = self.vi_count.saturating_add(1);
            }
        } else {
            self.vi_count = self.vi_count.saturating_add(1);
            self.header.length_vis = self.header.length_vis.max(self.vi_count);
        }
    }

    /// Emits a [`freeze::MovieFreeze`] suitable for serializing into a savestate.
    pub fn freeze(&self) -> MovieFreeze {
        freeze::v1::MovieFreeze {
            uid: self.header.uid,
            index: self.index.try_into().unwrap(),
            vi_count: self.vi_count.try_into().unwrap(),
            inputs: self.inputs.clone(),
        }
        .into()
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
    }

    ///
    pub fn load_freeze(&mut self, freeze: MovieFreeze) -> Result<(), M64PError> {
        match freeze {
            MovieFreeze::V1(freeze) => {
                if freeze.uid != self.header.uid {
                    return Err(M64PError::InputInvalid);
                }

                self.index = freeze.index.try_into().unwrap();
                self.vi_count = freeze.vi_count.try_into().unwrap();
                self.inputs = freeze.inputs;

                Ok(())
            }
            #[allow(unreachable_patterns)]
            _ => Err(M64PError::Incompatible),
        }
    }
}
