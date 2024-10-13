use std::{
    ffi::c_int,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use futures::{channel::oneshot, Future};
use m64prs_sys::CoreParam;

use crate::error::{M64PError, SavestateError};

/// Class that waits for a state change and resolves a savestate future.
pub(crate) struct SavestateWaiter {
    core_param: CoreParam,
    tx: oneshot::Sender<bool>,
}

/// Future implementation for savestates operations.
pub(crate) struct SavestateFuture {
    core_param: CoreParam,
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
            core_param: param,
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
