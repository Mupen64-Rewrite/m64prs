use std::{
    error::Error,
    ffi::c_int,
    fs, io,
    path::{Path, PathBuf},
};

use freeze::MovieFreeze;
use m64prs_core::{error::M64PError, Core};
use m64prs_sys::Buttons;
use movie::{M64File, M64Header, StartMethod};
use pollster::FutureExt;

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
}

impl VcrState {
    /// Initialize VCR for a new recording.
    pub fn new<P: Into<PathBuf>>(path: P, header: M64Header) -> Self {
        let path = path.into();
        Self {
            path,
            header,
            inputs: Vec::new(),
            index: 0,
            vi_count: 0,
            read_only: false,
        }
    }

    /// Loads an existing `.m64` file to record.
    pub fn load_m64<P: Into<PathBuf>>(path: P, read_only: bool) -> Result<Self, io::Error> {
        let path = path.into();
        let (header, inputs) = {
            let file = fs::File::open(&path)?;
            let M64File { header, inputs } = M64File::read_from(file)?;
            (header, inputs)
        };

        Ok(Self {
            path,
            header,
            inputs,
            index: 0,
            vi_count: 0,
            read_only,
        })
    }

    /// Saves the current VCR state to a `.m64` file.
    pub fn save_m64<P: AsRef<Path>>(&self, path: P) {
        todo!()
    }

    /// Resets all counters to frame 0 and sets up the core to restart playback.
    pub fn restart(&mut self, core: &Core) -> Result<(), Box<dyn Error>> {
        self.vi_count = 0;
        self.index = 0;

        match self.header.start_flags {
            StartMethod::FROM_RESET => core.reset(true)?,
            StartMethod::FROM_SNAPSHOT => {
                // search for a valid savestate (TODO: get rid of some of these unwraps)
                let st_path = fs::read_dir(self.path.parent().unwrap())?
                    .filter_map(|entry| {
                        let entry = entry.unwrap();
                        entry
                            .file_type()
                            .is_ok_and(|ty| ty.is_file())
                            .then(|| entry.path())
                    })
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, "No .st file found for .m64 movie")
                    })?;

                core.load_file(st_path).block_on()?;
            }
            StartMethod::FROM_EEPROM => {
                unimplemented!()
            }
            _ => panic!("invalid start flags"),
        }

        Ok(())
    }

    /// Implementation of [`m64prs_core::tas_callbacks::InputHandler`]. This method
    /// will either play back inputs (read/write mode), or overwrite inputs.
    pub fn filter_inputs(&mut self, _port: c_int, input: Buttons) -> Buttons {
        let index_usize: usize = self.index.try_into().unwrap();
        if self.read_only {
            if index_usize < self.inputs.len() {
                let result = self.inputs[index_usize];
                self.index += 1;
                result
            } else {
                input
            }
        } else {
            if index_usize < self.inputs.len() {
                self.inputs.truncate(index_usize);
            }
            self.inputs.push(input);
            self.index += 1;
            input
        }
    }

    /// Implementation of [`m64prs_core::tas_callbacks::FrameHandler`]. This method
    /// simply increments the VI count.
    pub fn tick_vi(&mut self) {
        if self.read_only {
            if usize::try_from(self.index).unwrap() < self.inputs.len() {
                self.vi_count = self.vi_count.saturating_add(1);
            }
        }
        else {
            self.vi_count = self.vi_count.saturating_add(1);
            self.header.length_vis = self.header.length_vis.max(self.vi_count);
        }
        
    }

    /// Determines whether the VCR has exhausted all inputs.
    pub fn reached_end(&self) -> bool {
        self.read_only && usize::try_from(self.index).unwrap() == self.inputs.len()
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
