use std::{collections::HashMap, ffi::c_int, os::raw::c_void};

use glutin::{
    config::{ColorBufferType, GetGlConfig, GlConfig},
    context::{GlContext, GlProfile, PossiblyCurrentContext, Version},
    display::{Display, GlDisplay},
    surface::{GlSurface, Surface, WindowSurface},
};

use m64prs_core::{
    ctypes::{self, GLAttribute, GLContextType, Size2D},
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
        gl_get_integer_v:
            unsafe extern "C" fn(pname: gl::types::GLenum, data: *mut gl::types::GLint),
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

    unsafe fn gl_get_attribute(attr: ctypes::GLAttribute) -> FFIResult<c_int> {
        let state = check_state_init!();

        if let GraphicsState::OpenGL {
            context,
            surface,
            gl_get_integer_v,
            ..
        } = &mut state.graphics
        {
            match attr {
                GLAttribute::DOUBLEBUFFER => Ok({
                    if surface.is_single_buffered() {
                        0
                    } else {
                        1
                    }
                }),
                GLAttribute::BUFFER_SIZE => Ok({
                    let config = context.config();

                    let color_size = match config.color_buffer_type() {
                        Some(ColorBufferType::Rgb {
                            r_size,
                            g_size,
                            b_size,
                        }) => r_size + g_size + b_size,
                        Some(ColorBufferType::Luminance(y_size)) => y_size,
                        None => 0,
                    };
                    let alpha_size = config.alpha_size();

                    (color_size as c_int) + (alpha_size as c_int)
                }),
                GLAttribute::DEPTH_SIZE => Ok({ context.config().depth_size() as c_int }),
                GLAttribute::RED_SIZE => match context.config().color_buffer_type() {
                    Some(ColorBufferType::Rgb { r_size, .. }) => Ok(r_size as c_int),
                    _ => Err(M64PError::SystemFail),
                },
                GLAttribute::GREEN_SIZE => match context.config().color_buffer_type() {
                    Some(ColorBufferType::Rgb { g_size, .. }) => Ok(g_size as c_int),
                    _ => Err(M64PError::SystemFail),
                },
                GLAttribute::BLUE_SIZE => match context.config().color_buffer_type() {
                    Some(ColorBufferType::Rgb { b_size, .. }) => Ok(b_size as c_int),
                    _ => Err(M64PError::SystemFail),
                },
                GLAttribute::ALPHA_SIZE => Ok(context.config().alpha_size() as c_int),
                GLAttribute::SWAP_CONTROL => Err(M64PError::Unsupported),
                GLAttribute::MULTISAMPLESAMPLES => Ok(context.config().num_samples() as c_int),
                GLAttribute::MULTISAMPLEBUFFERS => Ok({
                    if context.config().num_samples() > 0 {
                        1
                    } else {
                        0
                    }
                }),
                GLAttribute::CONTEXT_MAJOR_VERSION => match context.context_api() {
                    glutin::context::ContextApi::OpenGl(Some(Version { major, .. })) => {
                        Ok(major as c_int)
                    }
                    glutin::context::ContextApi::Gles(Some(Version { major, .. })) => {
                        Ok(major as c_int)
                    }
                    _ => Err(M64PError::SystemFail),
                },
                GLAttribute::CONTEXT_MINOR_VERSION => match context.context_api() {
                    glutin::context::ContextApi::OpenGl(Some(Version { minor, .. })) => {
                        Ok(minor as c_int)
                    }
                    glutin::context::ContextApi::Gles(Some(Version { minor, .. })) => {
                        Ok(minor as c_int)
                    }
                    _ => Err(M64PError::SystemFail),
                },
                GLAttribute::CONTEXT_PROFILE_MASK => match context.context_api() {
                    glutin::context::ContextApi::OpenGl(Some(Version { major, minor })) => {
                        if major > 3 || (major == 3 && minor >= 2) {
                            // OpenGL >= 3.2: query OpenGL for compatibility bit.
                            let mut profile_mask: gl::types::GLint = 0;
                            gl_get_integer_v(gl::CONTEXT_PROFILE_MASK, &mut profile_mask);
                            if ((profile_mask as gl::types::GLenum)
                                & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT)
                                != 0
                            {
                                Ok(GLContextType::COMPATIBILITY.0 as c_int)
                            } else {
                                Ok(GLContextType::CORE.0 as c_int)
                            }
                        } else {
                            // OpenGL < 3.2 always supports legacy functions.
                            Ok(GLContextType::COMPATIBILITY.0 as c_int)
                        }
                    }
                    glutin::context::ContextApi::Gles(Some(_)) => Ok(GLContextType::ES.0 as c_int),
                    _ => Err(M64PError::SystemFail),
                },
                _ => Err(M64PError::InputAssert),
            }
        } else {
            Err(M64PError::InvalidState)
        }
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
