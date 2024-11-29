use std::{
    ffi::{c_int, c_void},
    future::Future,
    pin::Pin,
    sync::{atomic::AtomicI32, mpsc, Arc},
    task::{Context, Poll},
};

use futures::channel::oneshot;
use m64prs_sys::{Command, CoreParam, EmuState};
use num_enum::TryFromPrimitive;

use crate::error::M64PError;

use super::Core;

// Asynchronous core commands
impl Core {
    /// Requests that the core stop execution of the current ROM.
    /// To await the core stopping, use [`Core::emu_state_change`]
    pub fn request_stop(&self) -> Result<(), M64PError> {
        self.do_command(Command::Stop)
    }
    /// Requests that the core pause execution of the current ROM.
    pub fn request_pause(&self) -> Result<(), M64PError> {
        self.do_command(Command::Pause)
    }
    /// Resumes the currently-running ROM.
    pub fn request_resume(&self) -> Result<(), M64PError> {
        self.do_command(Command::Resume)
    }
    /// Advances the currently-running ROM by one frame.
    pub fn request_advance_frame(&self) -> Result<(), M64PError> {
        self.do_command(Command::AdvanceFrame)
    }
    /// Resets the current ROM.
    pub fn reset(&self, hard: bool) -> Result<(), M64PError> {
        self.do_command_i(Command::Reset, hard as c_int)
    }

    /// Queries the emulator's current state.
    pub fn emu_state(&self) -> EmuState {
        unsafe {
            let mut result: c_int = 0;
            // SAFETY: the pointer cast is needed to pass the parameter.
            // This command should not do anything stupid like holding onto said pointer.
            self.do_command_ip(
                Command::CoreStateQuery,
                CoreParam::EmuState as c_int,
                &mut result as *mut _ as *mut c_void,
            )
            .expect("CoreDoCommand(M64CMD_CORE_STATE_QUERY, M64PARAM_EMU_STATE) should never fail");
            (result as <EmuState as TryFromPrimitive>::Primitive)
                .try_into()
                .unwrap()
        }
    }

    /// Async function that returns when the emulator changes state.
    pub fn emu_state_change(&self) -> impl Future<Output = EmuState> {
        let (future, waiter) = emu_pair();
        self.emu_sender.send(waiter)
            .expect("emu state wait queue should still be connected");

        future
    }
}

pub(crate) struct EmuStateWaiter {
    tx: oneshot::Sender<EmuState>,
}

pub(crate) struct EmuStateFuture {
    rx: oneshot::Receiver<EmuState>,
}

fn emu_pair() -> (EmuStateFuture, EmuStateWaiter) {
    let (tx, rx) = oneshot::channel();
    (EmuStateFuture { rx }, EmuStateWaiter { tx })
}

impl Future for EmuStateFuture {
    type Output = EmuState;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| res.unwrap())
    }
}

pub(crate) struct EmuStateWaitManager {
    rx: mpsc::Receiver<EmuStateWaiter>,
}
impl EmuStateWaitManager {
    pub fn new(rx: mpsc::Receiver<EmuStateWaiter>) -> Self {
        Self { rx }
    }

    pub fn on_state_change(&mut self, value: c_int) {
        let state = (value as <EmuState as TryFromPrimitive>::Primitive)
            .try_into()
            .unwrap();

        while let Ok(next) = self.rx.try_recv() {
            let _ = next.tx.send(state);
        }
    }
}
