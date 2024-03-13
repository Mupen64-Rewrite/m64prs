use std::{collections::HashMap, ffi::c_int, os::raw::c_void};

use glutin::{
    context::PossiblyCurrentContext,
    display::{Display, GlDisplay},
    surface::{GlSurface, Surface, WindowSurface},
};

use m64prs_core::{
    ctypes::{self, Size2D},
    error::M64PError,
    types::{FFIResult, VideoExtension},
};

use ash::vk;
use winit::{event_loop::EventLoop, window::Window};

mod opengl;

pub struct VidextState {
    event_loop: EventLoop<()>,
    graphics: GraphicsState,
}

enum GraphicsState {
    BuildOpenGL {
        attrs: HashMap<ctypes::GLAttribute, c_int>,
    },
    OpenGL {
        window: Window,
        display: Display,
        context: PossiblyCurrentContext,
        surface: Surface<WindowSurface>,
    },
}

static mut STATE: Option<VidextState> = None;

macro_rules! check_state_init {
    () => {
        match &mut STATE {
            Some(state) => state,
            None => return Err(M64PError::NotInit),
        }
    };
}

impl VideoExtension for VidextState {
    unsafe fn init_with_render_mode(mode: ctypes::RenderMode) -> FFIResult<()> {
        if STATE.is_some() {
            return Err(M64PError::AlreadyInit);
        }
        if mode == ctypes::RenderMode::VULKAN {
            return Err(M64PError::Unsupported);
        }

        let event_loop = EventLoop::new().unwrap();

        STATE = Some(VidextState {
            event_loop,
            graphics: GraphicsState::BuildOpenGL {
                attrs: HashMap::new(),
            },
        });

        Ok(())
    }

    unsafe fn quit() -> FFIResult<()> {
        if STATE.is_none() {
            return Err(M64PError::NotInit);
        }
        STATE = None;
        Ok(())
    }

    #[allow(refining_impl_trait)]
    unsafe fn list_fullscreen_modes() -> FFIResult<&'static [Size2D]> {
        Err(M64PError::Unsupported)
    }

    #[allow(refining_impl_trait)]
    unsafe fn list_fullscreen_rates(_size: ctypes::Size2D) -> FFIResult<&'static [c_int]> {
        Err(M64PError::Unsupported)
    }

    unsafe fn set_video_mode(
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: ctypes::VideoMode,
        flags: ctypes::VideoFlags,
    ) -> FFIResult<()> {
        let state = check_state_init!();

        if screen_mode == ctypes::VideoMode::FULLSCREEN
            || (flags & ctypes::VideoFlags::SUPPORT_RESIZING) != ctypes::VideoFlags(0)
        {
            return Err(M64PError::Unsupported);
        }

        opengl::setup_window(state, width, height, bits_per_pixel)?;

        Ok(())
    }

    unsafe fn set_video_mode_with_rate(
        _width: c_int,
        _height: c_int,
        _refresh_rate: c_int,
        _bits_per_pixel: c_int,
        _screen_mode: ctypes::VideoMode,
        _flags: ctypes::VideoFlags,
    ) -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn set_caption(_title: &std::ffi::CStr) -> FFIResult<()> {
        Ok(())
    }

    unsafe fn toggle_full_screen() -> FFIResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn resize_window(_width: c_int, _height: c_int) -> FFIResult<()> {
        Ok(())
    }

    unsafe fn gl_get_proc_address(symbol: &std::ffi::CStr) -> *mut std::ffi::c_void {
        let state = STATE.as_mut().unwrap();

        if let GraphicsState::OpenGL { display, .. } = &mut state.graphics {
            display.get_proc_address(symbol) as *mut c_void
        } else {
            panic!("VideoExtension::gl_get_proc_address called without GL initialized!");
        }
    }

    unsafe fn gl_set_attribute(attr: ctypes::GLAttribute, value: c_int) -> FFIResult<()> {
        let state = check_state_init!();

        if let GraphicsState::BuildOpenGL { attrs } = &mut state.graphics {
            attrs.insert(attr, value);
            Ok(())
        } else {
            Err(M64PError::InvalidState)
        }
    }

    unsafe fn gl_get_attribute(_attr: ctypes::GLAttribute) -> FFIResult<c_int> {
        todo!()
    }

    unsafe fn gl_swap_buffers() -> FFIResult<()> {
        let state = check_state_init!();

        if let GraphicsState::OpenGL {
            context, surface, ..
        } = &mut state.graphics
        {
            surface
                .swap_buffers(context)
                .map_err(|_| M64PError::SystemFail)
        } else {
            Err(M64PError::InvalidState)
        }
    }

    unsafe fn gl_get_default_framebuffer() -> u32 {
        0
    }

    unsafe fn vk_get_surface(_inst: vk::Instance) -> FFIResult<vk::SurfaceKHR> {
        todo!()
    }

    unsafe fn vk_get_instance_extensions() -> FFIResult<&'static [*const std::ffi::c_char]> {
        todo!()
    }
}
