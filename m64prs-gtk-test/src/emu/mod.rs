use std::{ops::{Deref, DerefMut}, path::Path, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}};

use m64prs_core::{error::StartupError, Core};

static CORE_INST: RwLock<Option<m64prs_core::Core>> = RwLock::new(None);

pub fn init_core<F>(path: &Path, f: F) -> Result<(), StartupError>
where
    F: FnOnce(&mut m64prs_core::Core) {
    let mut core = CORE_INST.write().expect("Failed to acquire lock");
    
    if core.is_some() {
        drop(core);
        panic!("Core is already initialized!");
    }

    let mut new_core = Core::init(path)?;
    f(&mut new_core);
    *core = Some(new_core);
    
    Ok(())
}

pub struct CoreHandle(RwLockReadGuard<'static, Option<Core>>);

impl Deref for CoreHandle {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

pub struct CoreHandleMut(RwLockWriteGuard<'static, Option<Core>>);

impl Deref for CoreHandleMut {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for CoreHandleMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

pub fn lock() -> CoreHandle {
    let core = CORE_INST.read().expect("Failed to acquire lock");
    
    if core.is_none() {
        drop(core);
        panic!("Core is not initialized!");
    }

    CoreHandle(core)
}

pub fn lock_mut() -> CoreHandleMut {
    let core = CORE_INST.write().expect("Failed to acquire lock");
    
    if core.is_none() {
        drop(core);
        panic!("Core is not initialized!");
    }

    CoreHandleMut(core)
}