use std::{
    collections::HashMap, ffi::{c_int, c_uint, CString}, num::NonZeroU32, os::raw::c_void, sync::RwLock, time::Duration
};

use glutin::{
    config::{Api, ColorBufferType, ConfigTemplateBuilder, GetGlConfig, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, GlContext, GlProfile, NotCurrentGlContext, Version},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use log::error;
use m64prs_core::{
    ctypes::{self, GLAttribute, GLContextType},
    error::M64PError,
    types::FFIResult,
};
use rwh_05::HasRawWindowHandle;
use winit::{dpi::PhysicalSize, event::{Event, WindowEvent}, platform::pump_events::EventLoopExtPumpEvents, window::WindowBuilder};

use super::{GraphicsState, VidextState};

pub fn setup_window(
    state: &mut VidextState,
    width: c_int,
    height: c_int,
    bits_per_pixel: c_int,
) -> FFIResult<()> {
    if let GraphicsState::BuildOpenGL { attrs: attr_map } = &mut state.graphics {
        log::debug!("Setting {}-bpp video mode @ {}x{}", bits_per_pixel, width, height);
        // error checking for width and height
        let width_nz = if width <= 0 {
            return Err(M64PError::InputInvalid);
        } else {
            unsafe { NonZeroU32::new_unchecked(width as u32) }
        };
        let height_nz = if height <= 0 {
            return Err(M64PError::InputInvalid);
        } else {
            unsafe { NonZeroU32::new_unchecked(height as u32) }
        };
        log::debug!("Got resolution {}x{}", width_nz, height_nz);

        // Setup the window and framebuffer config, we can't create a context without both
        let window_builder = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_resizable(false)
            .with_title("winit/vidext");
        let config_builder = gen_config_builder(bits_per_pixel, attr_map)?;

        // create the window and framebuffer config
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_builder(Some(window_builder))
            .build(&state.event_loop, config_builder, |mut iter| {
                // Ideally I want to propagate errors here but can't return result
                iter.next().unwrap()
            })
            .unwrap();
        let window = window.ok_or(M64PError::SystemFail)?;

        log::debug!("Constructed window and OpenGL config");

        // acquire the needed handles to create a context
        let window_handle = window.raw_window_handle();
        let gl_display = gl_config.display();

        // create the context
        let context_attrs = gen_context_builder(attr_map).build(Some(window_handle));
        let context = unsafe { gl_display.create_context(&gl_config, &context_attrs) }
            .map_err(|_| M64PError::SystemFail)?;

        log::debug!("Constructed OpenGL context");

        let surface_attrs = gen_surface_builder(attr_map).build(window_handle, width_nz, height_nz);
        let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attrs) }
            .map_err(|_| M64PError::SystemFail)?;

        log::debug!("Constructed OpenGL surface");

        let current_context = context
            .make_current(&surface)
            .map_err(|_| M64PError::SystemFail)?;

        log::debug!("Context is current");

        let gl_load_fn = |str: &str| -> *const c_void {
            let cstr = CString::new(str).unwrap();
            gl_display.get_proc_address(&cstr)
        };
        // load only GL functions that we need
        gl::GetIntegerv::load_with(gl_load_fn);
        gl::ClearColor::load_with(gl_load_fn);
        gl::Clear::load_with(gl_load_fn);

        // clear to black
        unsafe {
            gl::ClearColor(0.0f32, 0.0f32, 0.0f32, 0.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        state.graphics = GraphicsState::OpenGL {
            window: window,
            display: gl_display,
            context: current_context,
            surface: surface,
        };
        Ok(())
    } else {
        Err(M64PError::InvalidState)
    }
}

pub fn get_attribute(
    state: &mut VidextState,
    attr: ctypes::GLAttribute
) -> FFIResult<c_int> {
    if let GraphicsState::OpenGL {
        context,
        surface,
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
            GLAttribute::DEPTH_SIZE => Ok(context.config().depth_size() as c_int),
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
                        unsafe { gl::GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut profile_mask) };
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

pub fn swap_buffers(state: &mut VidextState, core: &RwLock<crate::CoreInner>) -> FFIResult<()> {
    if let GraphicsState::OpenGL {
        context, surface, ..
    } = &mut state.graphics
    {
        surface
            .swap_buffers(context)
            .map_err(|_| M64PError::SystemFail)?;

        // try to wake the event loop
        let _ = state.el_proxy.send_event(());
        // handle input
        state.event_loop.pump_events(Some(Duration::ZERO), |event, elwt| match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                print!("CLOSE RQ");
                let _ = core.read().unwrap().close_rom();
                elwt.exit();
            },
            _ => (),
        });

        Ok(())
    } else {
        Err(M64PError::InvalidState)
    }
}

fn gen_config_builder(
    bits_per_pixel: c_int,
    attr_map: &mut HashMap<GLAttribute, c_int>,
) -> FFIResult<ConfigTemplateBuilder> {
    let mut builder = ConfigTemplateBuilder::new();
    match bits_per_pixel {
        32 => {
            builder = builder
                .with_buffer_type(ColorBufferType::Rgb {
                    r_size: 8u8,
                    g_size: 8u8,
                    b_size: 8u8,
                })
                .with_alpha_size(8u8)
        }
        24 => {
            builder = builder
                .with_buffer_type(ColorBufferType::Rgb {
                    r_size: 8u8,
                    g_size: 8u8,
                    b_size: 8u8,
                })
                .with_alpha_size(0u8)
        }
        _ => (),
    }

    // BUFFER_SIZE overrides the BPP parameter
    if let Some(buffer_size) = attr_map.get(&GLAttribute::BUFFER_SIZE) {
        match buffer_size {
            32 => {
                builder = builder
                    .with_buffer_type(ColorBufferType::Rgb {
                        r_size: 8u8,
                        g_size: 8u8,
                        b_size: 8u8,
                    })
                    .with_alpha_size(8u8)
            }
            24 => {
                builder = builder
                    .with_buffer_type(ColorBufferType::Rgb {
                        r_size: 8u8,
                        g_size: 8u8,
                        b_size: 8u8,
                    })
                    .with_alpha_size(0u8)
            }
            _ => (),
        }
    }

    // RED_SIZE/GREEN_SIZE/BLUE_SIZE need to be specified together, they override
    // BUFFER_SIZE and bits_per_pixel. ALPHA_SIZE needs to be set to have alpha.
    if let (Some(red_size), Some(green_size), Some(blue_size)) = (
        attr_map.get(&GLAttribute::RED_SIZE),
        attr_map.get(&GLAttribute::GREEN_SIZE),
        attr_map.get(&GLAttribute::BLUE_SIZE),
    ) {
        builder = builder
            .with_buffer_type(ColorBufferType::Rgb {
                r_size: (*red_size).try_into().unwrap(),
                g_size: (*green_size).try_into().unwrap(),
                b_size: (*blue_size).try_into().unwrap(),
            })
            .with_alpha_size(0u8);
    }

    // ensure it's using GLES2 if they asked for it
    let profile = attr_map
        .get(&GLAttribute::CONTEXT_PROFILE_MASK)
        .map(|profile| GLContextType(*profile as u32));
    if let Some(GLContextType::ES) = profile {
        builder = builder.with_api(Api::GLES2 | Api::GLES3);
    } else {
        builder = builder.with_api(Api::OPENGL);
    }

    // other config-dependent attributes
    if let Some(double_buffer) = attr_map.get(&GLAttribute::DOUBLEBUFFER) {
        let is_single_buffer = *double_buffer == 0;
        if is_single_buffer {
            // if the plugin expects single buffering it might not call
            // swap_buffers, so it's going to fuck things up for us
            error!("Single buffering is not supported");
            return Err(M64PError::Unsupported);
        }
    }
    if let Some(depth_size) = attr_map.get(&GLAttribute::DEPTH_SIZE) {
        builder = builder.with_depth_size((*depth_size).try_into().unwrap());
    }
    if let Some(alpha_size) = attr_map.get(&GLAttribute::ALPHA_SIZE) {
        builder = builder.with_alpha_size((*alpha_size).try_into().unwrap());
    }
    if let Some(swap_interval) = attr_map.get(&GLAttribute::SWAP_CONTROL) {
        builder = builder.with_swap_interval(Some((*swap_interval).try_into().unwrap()), None);
    }
    if let Some(samples) = attr_map.get(&GLAttribute::MULTISAMPLESAMPLES) {
        builder = builder.with_multisampling((*samples).try_into().unwrap());
    }

    Ok(builder)
}

fn gen_context_builder(attr_map: &mut HashMap<GLAttribute, c_int>) -> ContextAttributesBuilder {
    let major_version = attr_map
        .get(&GLAttribute::CONTEXT_MAJOR_VERSION)
        .map(|x| *x)
        .unwrap_or(3);
    let minor_version = attr_map
        .get(&GLAttribute::CONTEXT_MINOR_VERSION)
        .map(|x| *x)
        .unwrap_or(3);
    let profile = attr_map
        .get(&GLAttribute::CONTEXT_PROFILE_MASK)
        .map(|profile| GLContextType(*profile as c_uint))
        .unwrap_or(GLContextType::COMPATIBILITY);

    let mut builder = ContextAttributesBuilder::new();

    if profile == GLContextType::ES {
        builder = builder.with_context_api(ContextApi::Gles(Some(Version {
            major: major_version.try_into().unwrap(),
            minor: minor_version.try_into().unwrap(),
        })));
    } else {
        builder = builder.with_context_api(ContextApi::OpenGl(Some(Version {
            major: major_version.try_into().unwrap(),
            minor: minor_version.try_into().unwrap(),
        })));
        builder = match profile {
            GLContextType::COMPATIBILITY => builder.with_profile(GlProfile::Compatibility),
            GLContextType::CORE => builder.with_profile(GlProfile::Core),
            _ => builder.with_profile(GlProfile::Compatibility),
        }
    }

    builder
}

fn gen_surface_builder(
    attr_map: &mut HashMap<GLAttribute, c_int>,
) -> SurfaceAttributesBuilder<WindowSurface> {
    let mut builder = SurfaceAttributesBuilder::<WindowSurface>::new();

    builder = builder.with_single_buffer(false);

    if let Some(double_buffer) = attr_map.get(&GLAttribute::DOUBLEBUFFER) {
        let is_single_buffer = *double_buffer == 0;
        builder = builder.with_single_buffer(is_single_buffer);
    }
    builder
}