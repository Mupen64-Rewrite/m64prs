use std::{
    ffi::c_int,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use futures::{channel::oneshot, Future, FutureExt};

use crate::ctypes;

/// Future implementation for savestates operations.
pub struct SavestateFuture {
    rx: oneshot::Receiver<bool>,
}

impl Future for SavestateFuture {
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Future::poll(Pin::new(&mut self.rx), cx) {
            Poll::Ready(res) => Poll::Ready(res.unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl SavestateFuture {
    pub(super) fn new(rx: oneshot::Receiver<bool>) -> Self {
        Self { rx }
    }
}

/// Class that waits for a state change and resolves a savestate future.
pub(super) struct SavestateWaiter {
    pub param: ctypes::CoreParam,
    pub tx: oneshot::Sender<bool>,
}

pub(super) struct SavestateWaitManager {
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

    pub fn on_state_change(&mut self, param: ctypes::CoreParam, value: c_int) {
        // add any new waiters that may need to be processed
        while let Ok(next) = self.rx.try_recv() {
            self.waiters.push(next);
        }

        // if any waiters need to be tripped, trip them now and remove them.
        let mut i = 0;
        while i < self.waiters.len() {
            if self.waiters[i].param == param {
                let waiter = self.waiters.swap_remove(i);
                waiter.tx.send(value != 0).unwrap();
            } else {
                i += 1;
            }
        }
    }
}
