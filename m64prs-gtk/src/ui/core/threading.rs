use m64prs_core::{error::M64PError, Core};
use std::sync::Weak;
use std::{
    ops::Deref,
    sync::Arc,
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub(super) struct RunningCore(Option<Inner>);

#[derive(Debug)]
struct Inner {
    core: Arc<Core>,
    join_handle: JoinHandle<Result<(), M64PError>>,
}

impl RunningCore {
    pub(super) fn execute(core: Core) -> Self {
        let core = Arc::new(core);
        let join_handle = {
            let core = Arc::clone(&core);
            thread::spawn(move || {
                let result = core.execute();
                result
            })
        };
        Self(Some(Inner { core, join_handle }))
    }

    pub(super) fn stop(mut self) -> (Core, Result<(), M64PError>) {
        self.0.take().unwrap().stop()
    }
}

impl Drop for RunningCore {
    fn drop(&mut self) {
        if let Some(inner) = self.0.take() {
            let _ = inner.stop();
        }
    }
}

impl Deref for RunningCore {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &*self.0.as_ref().unwrap().core
    }
}

impl Inner {
    fn stop(self) -> (Core, Result<(), M64PError>) {
        let _ = self.core.request_stop();
        let result = self.join_handle.join().unwrap();
        let core = Arc::into_inner(self.core).expect("this should be the only reference");
        (core, result)
    }
}
