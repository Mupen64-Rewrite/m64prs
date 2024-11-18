use std::{
    ffi::{c_int, c_void},
    future::Future,
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use futures::channel::oneshot;
use m64prs_sys::{Command, CoreParam, EmuState};

use crate::error::M64PError;

use super::Core;

// Asynchronous core commands
impl Core {
    /// Stops the currently-running ROM.
    pub async fn stop(&self) -> Result<(), M64PError> {
        let _lock = self.emu_mutex.lock().await;
        self.emu_state_command(Command::Stop, EmuState::Stopped)
            .await
    }
    /// Pauses the currently-running ROM.
    pub async fn pause(&self) -> Result<(), M64PError> {
        let _lock = self.emu_mutex.lock().await;
        self.emu_state_command(Command::Pause, EmuState::Paused)
            .await
    }
    /// Resumes the currently-running ROM.
    pub async fn resume(&self) -> Result<(), M64PError> {
        let _lock = self.emu_mutex.lock().await;
        self.emu_state_command(Command::Resume, EmuState::Running)
            .await
    }
    /// Advances the currently-running ROM by one frame.
    pub async fn advance_frame(&self) -> Result<(), M64PError> {
        let _lock = self.emu_mutex.lock().await;
        self.emu_state_command(Command::AdvanceFrame, EmuState::Paused)
            .await
    }

    /// Resets the current ROM.
    pub fn reset(&self) -> Result<(), M64PError> {
        self.do_command_i(Command::Reset, 1)
    }

    /// Waits until the emulator state changes to a desired value.
    pub async fn await_emu_state(&self, state: EmuState) {
        let _lock = self.emu_mutex.lock().await;
        self.await_emu_state_inner(state).await.unwrap();
    }

    /// Notifies the graphics plugin of a change in the window's size.
    pub fn notify_resize(&self, width: u16, height: u16) -> Result<(), M64PError> {
        let size_packed = (((width as u32) << 16) | (height as u32)) as c_int;
        unsafe {
            // SAFETY: The pointer to the 32-bit size passed here is only
            // used during the call to CoreStateSet.
            self.do_command_ip(
                Command::CoreStateSet,
                CoreParam::VideoSize as c_int,
                &size_packed as *const c_int as *mut c_void,
            )
        }
    }

    fn emu_state_command(
        &self,
        command: Command,
        state: EmuState,
    ) -> impl Future<Output = Result<(), M64PError>> {
        let (mut future, waiter) = emu_pair(state as c_int);
        self.emu_sender
            .send(waiter)
            .expect("emu waiter queue should still be connected");

        if let Err(error) = self.do_command(command) {
            future.fail_early(error);
        }

        unsafe {
            let mut cur_state: i32 = 0;
            if let Err(error) = self.do_command_ip(
                Command::CoreStateQuery,
                CoreParam::EmuState as i32,
                &mut cur_state as *mut _ as *mut _,
            ) {
                future.fail_early(error);
                return future;
            }
            if cur_state == state as c_int {
                future.succeed_early();
                return future;
            }
        }

        future
    }

    fn await_emu_state_inner(
        &self,
        state: EmuState,
    ) -> impl Future<Output = Result<(), M64PError>> {
        let (mut future, waiter) = emu_pair(state as c_int);
        self.emu_sender
            .send(waiter)
            .expect("emu waiter queue should still be connected");

        unsafe {
            let mut cur_state: i32 = 0;
            if let Err(error) = self.do_command_ip(
                Command::CoreStateQuery,
                CoreParam::EmuState as i32,
                &mut cur_state as *mut _ as *mut _,
            ) {
                future.fail_early(error);
                return future;
            }
            if cur_state == state as c_int {
                future.succeed_early();
                return future;
            }
        }

        future
    }
}

pub(crate) struct EmulatorWaiter {
    value: c_int,
    tx: oneshot::Sender<()>,
}

pub(crate) struct EmulatorFuture {
    early_result: Option<Result<(), M64PError>>,
    rx: oneshot::Receiver<()>,
}

impl Future for EmulatorFuture {
    type Output = Result<(), M64PError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.early_result.take() {
            return Poll::Ready(result);
        }

        match Future::poll(Pin::new(&mut self.rx), cx) {
            Poll::Ready(_) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl EmulatorFuture {
    pub(crate) fn fail_early(&mut self, error: M64PError) {
        self.early_result = Some(Err(error));
    }

    pub(crate) fn succeed_early(&mut self) {
        self.early_result = Some(Ok(()))
    }
}

fn emu_pair(value: c_int) -> (EmulatorFuture, EmulatorWaiter) {
    let (tx, rx) = oneshot::channel();
    (
        EmulatorFuture {
            early_result: None,
            rx,
        },
        EmulatorWaiter { value, tx },
    )
}

pub(crate) struct EmulatorWaitManager {
    rx: mpsc::Receiver<EmulatorWaiter>,
    waiters: Vec<EmulatorWaiter>,
}
impl EmulatorWaitManager {
    pub fn new(rx: mpsc::Receiver<EmulatorWaiter>) -> Self {
        Self {
            rx,
            waiters: Vec::new(),
        }
    }

    pub fn on_emu_state_changed(&mut self, value: c_int) {
        // add any new waiters that may need to be processed
        while let Ok(next) = self.rx.try_recv() {
            self.waiters.push(next);
        }

        // if any waiters need to be tripped, trip them now and remove them.
        let mut i = 0;
        while i < self.waiters.len() {
            if self.waiters[i].value == value {
                let waiter = self.waiters.swap_remove(i);
                let _ = waiter.tx.send(());
            } else {
                i += 1;
            }
        }
    }
}
