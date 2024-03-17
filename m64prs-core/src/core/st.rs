use std::{
    ffi::c_int,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use futures::{channel::oneshot, Future, FutureExt};

use crate::ctypes;

pub(super) struct SavestateFuture {
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
    fn new(rx: oneshot::Receiver<bool>) -> Self {
        Self { rx }
    }
}

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
        while let Ok(next) = self.rx.try_recv() {
            self.waiters.push(next);
        }

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
