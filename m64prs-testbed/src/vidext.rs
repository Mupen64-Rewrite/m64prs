use m64prs_core::{
    error::M64PError,
    types::{FFIResult, VideoExtension},
    Core,
};
use m64prs_sys::{GLAttribute, RenderMode, Size2D, VideoFlags, VideoMode};
use send_wrapper::SendWrapper;
use std::{iter, ptr::null_mut, sync::RwLock};

use std::{
    cell::{RefCell, RefMut},
    ffi::{c_char, c_int, c_void, CStr},
    sync::{Arc, OnceLock},
};
use winit::event_loop::{EventLoop, EventLoopProxy};

mod opengl;
use opengl::*;

enum VideoUserEvent {
    CoreQuitRequest,
}

pub struct VideoState {
    core: Arc<RwLock<Core>>,
    event_loop: EventLoop<VideoUserEvent>,
    event_proxy: EventLoopProxy<VideoUserEvent>,
    render: RenderState,
}

impl VideoState {
    pub fn new(core: Arc<RwLock<Core>>) -> Self {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        let event_proxy = event_loop.create_proxy();

        Self { core, event_loop, event_proxy, render: RenderState::Uninit }
    }
}

enum RenderState {
    Uninit,
    OpenGl(OpenGlState),
}

impl Default for RenderState {
    fn default() -> Self {
        RenderState::Uninit
    }
}

impl VideoExtension for VideoState {
    unsafe fn init_with_render_mode(&mut self, mode: RenderMode) -> FFIResult<()> {
        match self.render {
            RenderState::Uninit => {
                match mode {
                    RenderMode::OpenGl => self.render = RenderState::OpenGl(OpenGlState::init()),
                    RenderMode::Vulkan => return Err(M64PError::Unsupported),
                }
                Ok(())
            }
            _ => Err(M64PError::AlreadyInit),
        }
    }

    unsafe fn quit(&mut self) -> FFIResult<()> {
        match &mut self.render {
            RenderState::Uninit => Err(M64PError::NotInit),
            RenderState::OpenGl(opengl_state) => {
                self.event_proxy
                    .send_event(VideoUserEvent::CoreQuitRequest)
                    .map_err(|_| M64PError::SystemFail)?;
                opengl_state.cleanup(&mut self.event_loop);
                Ok(())
            }
        }
    }
    
    unsafe fn list_fullscreen_modes(&mut self) -> FFIResult<impl Iterator<Item = Size2D>> {
        Result::<iter::Empty<Size2D>, M64PError>::Err(M64PError::Unsupported)
    }

    unsafe fn list_fullscreen_rates(&mut self, _size: Size2D) -> FFIResult<impl Iterator<Item = c_int>> {
        Result::<iter::Empty<c_int>, M64PError>::Err(M64PError::Unsupported)
    }

    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()> {
        match &mut self.render {
            RenderState::Uninit => Err(M64PError::NotInit),
            RenderState::OpenGl(opengl_state) => opengl_state.set_video_mode(
                Arc::clone(&self.core),
                &mut self.event_loop,
                width,
                height,
                bits_per_pixel,
                screen_mode,
                flags,
            ),
        }
    }

    unsafe fn set_video_mode_with_rate(
        &mut self,
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn set_caption(&mut self, title: &CStr) -> FFIResult<()> {
        Ok(())
    }

    unsafe fn toggle_full_screen(&mut self) -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn resize_window(&mut self, width: c_int, height: c_int) -> FFIResult<()> {
        match &mut self.render {
            RenderState::Uninit => Err(M64PError::NotInit),
            RenderState::OpenGl(opengl_state) => opengl_state.resize_window(width, height),
        }
    }

    unsafe fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void {
        match &mut self.render {
            RenderState::Uninit => null_mut(),
            RenderState::OpenGl(opengl_state) => opengl_state.gl_get_proc_address(symbol),
        }
    }

    unsafe fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> FFIResult<()> {
        match &mut self.render {
            RenderState::Uninit => Err(M64PError::NotInit),
            RenderState::OpenGl(opengl_state) => opengl_state.gl_set_attribute(attr, value),
        }
    }

    unsafe fn gl_get_attribute(&mut self, attr: GLAttribute) -> FFIResult<c_int> {
        match &mut self.render {
            RenderState::Uninit => Err(M64PError::NotInit),
            RenderState::OpenGl(opengl_state) => opengl_state.gl_get_attribute(attr),
        }
    }

    unsafe fn gl_swap_buffers(&mut self) -> FFIResult<()> {
        match &mut self.render {
            RenderState::Uninit => Err(M64PError::NotInit),
            RenderState::OpenGl(opengl_state) => opengl_state.gl_swap_buffers(&mut self.event_loop),
        }
    }

    unsafe fn gl_get_default_framebuffer(&mut self) -> u32 {
        0
    }

    unsafe fn vk_get_surface(&mut self, inst: ash::vk::Instance) -> FFIResult<ash::vk::SurfaceKHR> {
        Err(M64PError::Unsupported)
    }

    unsafe fn vk_get_instance_extensions(&mut self) -> FFIResult<&'static [*const c_char]> {
        Err(M64PError::Unsupported)
    }
}