use std::{
    env, fs, mem,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use m64prs_core::{plugin::PluginSet, Plugin};
use relm4::{tokio, ComponentSender, Worker};

use crate::ui::main;

#[derive(Debug)]
pub enum Request {
    Init,
}

#[derive(Debug)]
pub enum Update {
    CoreReady
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkerState {
    Uninit = 0,
    Ready = 1,
    Paused = 2,
    Running = 3,
}

#[derive(Debug)]
enum ModelInner {
    Uninit,
    Ready(m64prs_core::Core),
    Running {
        join_handle: JoinHandle<()>,
        core_ref: Arc<RwLock<m64prs_core::Core>>,
    },
}

#[derive(Debug)]
pub struct Model(ModelInner);

impl Model {
    fn init(&mut self, sender: &ComponentSender<Self>) {
        #[cfg(target_os = "windows")]
        const MUPEN_FILENAME: &str = "mupen64plus.dll";
        #[cfg(target_os = "linux")]
        const MUPEN_FILENAME: &str = "libmupen64plus.so";

        self.0 = if let ModelInner::Uninit = self.0 {
            let self_path = env::current_exe().expect("should be able to find current_exe");
            let mupen_dll_path = self_path
                .parent()
                .map(|path| path.join(MUPEN_FILENAME))
                .expect("should be able to access other file in the same folder");

            let core =
                m64prs_core::Core::init(mupen_dll_path).expect("core startup should succeed");

            ModelInner::Ready(core)
        } else {
            panic!("core is already initialized");
        };
        sender.output(Update::CoreReady).unwrap();
    }
}

impl Worker for Model {
    type Init = ();

    type Input = Request;
    type Output = Update;

    fn init(_: Self::Init, sender: ComponentSender<Self>) -> Self {
        let result = Self(ModelInner::Uninit);
        sender.input(Request::Init);
        result
    }

    fn update(&mut self, request: Self::Input, sender: ComponentSender<Self>) {
        match request {
            Request::Init => self.init(&sender)
        }
    }
}
