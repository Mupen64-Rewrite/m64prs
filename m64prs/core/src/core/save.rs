use std::{
    ffi::{c_int, CString},
    path::Path,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use futures::{channel::oneshot, Future};
use m64prs_sys::{Command, CoreParam};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::{M64PError, SavestateError};

use super::Core;

/// Functions dealing with savestates.
impl Core {
    /// Saves game state to the current slot.
    pub async fn save_slot(&self) -> Result<(), SavestateError> {
        let _lock = self.st_mutex.lock().await;
        self.save_op_inner(CoreParam::StateSaveComplete, || {
            self.do_command(Command::StateSave)
        })
        .await
    }

    /// Loads game state from the current slot.
    pub async fn load_slot(&self) -> Result<(), SavestateError> {
        let _lock = self.st_mutex.lock().await;
        self.save_op_inner(CoreParam::StateLoadComplete, || {
            self.do_command(Command::StateLoad)
        })
        .await
    }

    pub async fn save_file<P: AsRef<Path>>(
        &self,
        path: P,
        format: SavestateFormat,
    ) -> Result<(), SavestateError> {
        let _lock = self.st_mutex.lock().await;
        let c_path = CString::new(path.as_ref().to_str().unwrap()).unwrap();

        self.save_op_inner(CoreParam::StateSaveComplete, || unsafe {
            self.do_command_ip(
                Command::StateSave,
                format as c_int,
                c_path.as_ptr() as *mut _,
            )
        })
        .await
    }

    pub async fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SavestateError> {
        let _lock = self.st_mutex.lock().await;
        let c_path = CString::new(path.as_ref().to_str().unwrap()).unwrap();

        self.save_op_inner(CoreParam::StateLoadComplete, || unsafe {
            self.do_command_p(Command::StateLoad, c_path.as_ptr() as *mut _)
        })
        .await
    }

    fn save_op_inner<F: FnOnce() -> Result<(), M64PError>>(
        &self,
        param: CoreParam,
        f: F,
    ) -> impl Future<Output = Result<(), SavestateError>> {
        let (mut future, waiter) = save_pair(param);
        self.st_sender
            .send(waiter)
            .expect("save waiter queue should still be connected");
        // initiate the save operation. This is guaranteed to trip the waiter at some point.
        if let Err(error) = f() {
            future.fail_early(error);
        }

        future
    }

    pub fn set_state_slot(&self, slot: u8) -> Result<(), M64PError> {
        if slot > 9 {
            panic!("Slot value must be between 0-9")
        }

        self.do_command_i(Command::StateSetSlot, slot as i32)
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum SavestateFormat {
    Mupen64Plus = 1,
    Project64 = 2,
    Project64Uncompressed = 3,
}

/// Class that waits for a state change and resolves a savestate future.
pub(crate) struct SavestateWaiter {
    core_param: CoreParam,
    tx: oneshot::Sender<bool>,
}

/// Future implementation for savestates operations.
pub(crate) struct SavestateFuture {
    early_fail: Option<M64PError>,
    rx: oneshot::Receiver<bool>,
}

impl Future for SavestateFuture {
    type Output = Result<(), SavestateError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(err) = self.early_fail.take() {
            return Poll::Ready(Err(SavestateError::EarlyFail(err)));
        }

        match Future::poll(Pin::new(&mut self.rx), cx) {
            Poll::Ready(res) => {
                if res.unwrap() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Ready(Err(SavestateError::SaveLoad))
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl SavestateFuture {
    pub(crate) fn fail_early(&mut self, error: M64PError) {
        self.early_fail = Some(error);
    }
}

pub(crate) fn save_pair(param: CoreParam) -> (SavestateFuture, SavestateWaiter) {
    let (tx, rx) = oneshot::channel();
    (
        SavestateFuture {
            early_fail: None,
            rx,
        },
        SavestateWaiter {
            core_param: param,
            tx,
        },
    )
}

pub(crate) struct SavestateWaitManager {
    rx: mpsc::Receiver<SavestateWaiter>,
    waiters: Vec<SavestateWaiter>,
}
impl SavestateWaitManager {
    pub fn new(rx: mpsc::Receiver<SavestateWaiter>) -> Self {
        Self {
            rx,
            waiters: Vec::new(),
        }
    }

    pub fn on_state_change(&mut self, param: CoreParam, value: c_int) {
        // add any new waiters that may need to be processed
        while let Ok(next) = self.rx.try_recv() {
            self.waiters.push(next);
        }

        // if any waiters need to be tripped, trip them now and remove them.
        let mut i = 0;
        while i < self.waiters.len() {
            if self.waiters[i].core_param == param {
                let waiter = self.waiters.swap_remove(i);
                let _ = waiter.tx.send(value != 0);
            } else {
                i += 1;
            }
        }
    }
}
