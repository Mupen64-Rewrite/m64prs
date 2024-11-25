use std::{
    ffi::c_int,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use futures::{channel::oneshot, Future};
use m64prs_sys::{Command, CoreParam};

use crate::error::{M64PError, SavestateError};

use super::Core;

/// Functions dealing with savestates.
impl Core {
    /// Saves game state to the current slot.
    pub async fn save_state(&self) -> Result<(), SavestateError> {
        let _lock = self.st_mutex.lock().await;
        let res = self.save_state_inner().await;
        res
    }

    /// Loads game state from the current slot.
    pub async fn load_state(&self) -> Result<(), SavestateError> {
        let _lock = self.st_mutex.lock().await;
        let res = self.load_state_inner().await;
        res
    }

    fn save_state_inner(&self) -> impl Future<Output = Result<(), SavestateError>> {
        // create transmission channel for savestate result
        let (mut future, waiter) = save_pair(CoreParam::StateSaveComplete);
        self.st_sender
            .send(waiter)
            .expect("save waiter queue should still be connected");
        // initiate the save operation. This is guaranteed to trip the waiter at some point.
        if let Err(error) = self.do_command(Command::StateSave) {
            future.fail_early(error);
        }

        future
    }

    fn load_state_inner(&self) -> impl Future<Output = Result<(), SavestateError>> {
        let (mut future, waiter) = save_pair(CoreParam::StateLoadComplete);
        self.st_sender
            .send(waiter)
            .expect("save waiter queue should still be connected");

        if let Err(error) = self.do_command(Command::StateSave) {
            future.fail_early(error);
        }

        future
    }

    pub fn set_savestate_slot(&self, slot: u8) -> Result<(), M64PError> {
        if slot > 9 {
            panic!("Slot value must be between 0-9")
        }

        self.do_command_i(Command::StateSetSlot, slot as i32)
    }
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
