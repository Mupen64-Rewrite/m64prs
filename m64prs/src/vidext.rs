use std::sync::Mutex;
use m64prs_core::{
    error::M64PError,
    types::{FFIResult, VideoExtension}, vidext_table,
};
use m64prs_sys::{GLAttribute, RenderMode, Size2D, VideoFlags, VideoMode};
use std::{
    cell::RefCell,
    ffi::{c_char, c_int, c_void, CStr},
    sync::OnceLock,
    thread::{self, ThreadId},
};
use winit::event_loop::{EventLoop, EventLoopBuilder};

mod opengl;
use opengl::*;

/// A thread-safe, overly-protective wrapper around the event loop.
struct SafeEventLoop<T: 'static> {
    event_loop: RefCell<EventLoop<T>>,
    src_thread: ThreadId,
}

impl<T: 'static> SafeEventLoop<T> {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::<T>::with_user_event()
            .build()
            .expect("Event loop creation failed!");
        let src_thread = thread::current().id();

        Self {
            event_loop: RefCell::new(event_loop),
            src_thread,
        }
    }

    pub fn is_src_thread(&self) -> bool {
        thread::current().id() != self.src_thread
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut EventLoop<T>) -> R) -> Result<R, ()> {
        if thread::current().id() != self.src_thread {
            Err(())
        } else {
            Ok(f(&mut self.event_loop.borrow_mut()))
        }
    }
}

unsafe impl<T> Send for SafeEventLoop<T> {}
unsafe impl<T> Sync for SafeEventLoop<T> {}

static EVENT_LOOP: OnceLock<SafeEventLoop<()>> = OnceLock::new();

enum VideoState {
    Uninit,
    OpenGlInit(OpenGlInitState),
    OpenGlActive(OpenGlActiveState),
}

unsafe impl Send for VideoState {}
unsafe impl Sync for VideoState {}

impl VideoExtension for VideoState {
    unsafe fn init_with_render_mode(&mut self, mode: RenderMode) -> FFIResult<()> {
        match self {
            VideoState::Uninit => {
                let _event_loop = EVENT_LOOP.get_or_init(|| SafeEventLoop::new());

                match mode {
                    RenderMode::OpenGl => {
                        *self = VideoState::OpenGlInit(OpenGlInitState::default())
                    }
                    RenderMode::Vulkan => return Err(M64PError::Unsupported),
                }
            }
            _ => return Err(M64PError::AlreadyInit)
        }

        Ok(())
    }

    unsafe fn quit(&mut self) -> FFIResult<()> {
        match self {
            VideoState::Uninit => return Err(M64PError::NotInit),
            _ => *self = VideoState::Uninit,
        }
        Ok(())
    }

    unsafe fn list_fullscreen_modes(&mut self) -> FFIResult<&'static [Size2D]> {
        Err(M64PError::Unsupported)
    }

    unsafe fn list_fullscreen_rates(&mut self, _size: Size2D) -> FFIResult<&'static [c_int]> {
        Err(M64PError::Unsupported)
    }

    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        video_flags: VideoFlags,
    ) -> FFIResult<()> {
        *self = match self {
            VideoState::OpenGlInit(state) => {
                VideoState::OpenGlActive(OpenGlActiveState::init_from(
                    state,
                    OpenGlInitParams {
                        width,
                        height,
                        bits_per_pixel,
                        screen_mode,
                        video_flags,
                    },
                )?)
            }
            VideoState::Uninit => return Err(M64PError::NotInit),
            _ => return Err(M64PError::InvalidState),
        };
        Ok(())
    }

    unsafe fn set_video_mode_with_rate(
        &mut self,
        _width: c_int,
        _height: c_int,
        _refresh_rate: c_int,
        _bits_per_pixel: c_int,
        _screen_mode: VideoMode,
        _flags: VideoFlags,
    ) -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn set_caption(&mut self, _title: &CStr) -> FFIResult<()> {
        // pretend it happened
        Ok(())
    }

    unsafe fn toggle_full_screen(&mut self) -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn resize_window(&mut self, _width: c_int, _height: c_int) -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void {
        match self {
            VideoState::OpenGlActive(state) => state.get_proc_address(symbol),
            _ => panic!("No available OpenGL context for VidExt_GL_GetProcAddress"),
        }
    }

    unsafe fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> FFIResult<()> {
        match self {
            VideoState::OpenGlInit(state) => state.set_attr(attr, value),
            VideoState::Uninit => return Err(M64PError::NotInit),
            _ => return Err(M64PError::InvalidState),
        }
    }

    unsafe fn gl_get_attribute(&mut self, attr: GLAttribute) -> FFIResult<c_int> {
        match self {
            VideoState::OpenGlActive(state) => state.get_attr(attr),
            VideoState::Uninit => return Err(M64PError::NotInit),
            _ => return Err(M64PError::InvalidState),
        }
    }

    unsafe fn gl_swap_buffers(&mut self) -> FFIResult<()> {
        match self {
            VideoState::OpenGlActive(state) => state.swap_buffers(),
            VideoState::Uninit => return Err(M64PError::NotInit),
            _ => return Err(M64PError::InvalidState),
        }
    }

    unsafe fn gl_get_default_framebuffer(&mut self) -> u32 {
        0
    }

    unsafe fn vk_get_surface(
        &mut self,
        _inst: ash::vk::Instance,
    ) -> FFIResult<ash::vk::SurfaceKHR> {
        Err(M64PError::Unsupported)
    }

    unsafe fn vk_get_instance_extensions(&mut self) -> FFIResult<&'static [*const c_char]> {
        Err(M64PError::Unsupported)
    }
}

static VIDEXT_INST: Mutex<VideoState> = Mutex::new(VideoState::Uninit);

vidext_table!([VIDEXT_INST] pub VIDEXT_TABLE);