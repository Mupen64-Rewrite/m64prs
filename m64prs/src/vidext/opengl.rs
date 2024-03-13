use std::{
    collections::HashMap,
    ffi::{c_int, c_uint},
    mem,
    num::NonZeroU32,
};

use glutin::{
    config::{Api, ColorBufferType, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext, Version},
    display::{GetGlDisplay, GlDisplay},
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use m64prs_core::{
    ctypes::{GLAttribute, GLContextType},
    error::M64PError,
    types::FFIResult,
};
use rwh_05::HasRawWindowHandle;
use winit::{dpi::PhysicalSize, window::WindowBuilder};

use super::{GraphicsState, VidextState};

use cstr::cstr;

pub fn setup_window(
    state: &mut VidextState,
    width: c_int,
    height: c_int,
    bits_per_pixel: c_int,
) -> FFIResult<()> {
    if let GraphicsState::BuildOpenGL { attrs: attr_map } = &mut state.graphics {
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

        // Setup the window and framebuffer config, we can't create a context without both
        let window_builder = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_resizable(false)
            .with_title("winit/vidext");
        let config_builder = gen_config_builder(bits_per_pixel, attr_map);

        // create the window and framebuffer config
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_builder(Some(window_builder))
            .build(&state.event_loop, config_builder, |mut iter| {
                // Ideally I want to propagate errors here but can't return result
                iter.next().unwrap()
            })
            .unwrap();
        let window = window.ok_or(M64PError::SystemFail)?;

        // acquire the needed handles to create a context
        let window_handle = window.raw_window_handle();
        let gl_display = gl_config.display();

        // create the context
        let context_attrs = gen_context_builder(attr_map).build(Some(window_handle));
        let context = unsafe { gl_display.create_context(&gl_config, &context_attrs) }
            .map_err(|_| M64PError::SystemFail)?;

        let surface_attrs = gen_surface_builder(attr_map).build(window_handle, width_nz, height_nz);
        let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attrs) }
            .map_err(|_| M64PError::SystemFail)?;

        let current_context = context
            .make_current(&surface)
            .map_err(|_| M64PError::SystemFail)?;
        
        // We could use an OpenGL loading library, but that's too much overhead
        // This is needed to distinguish core from compatiblity on OpenGL (not ES).
        let gl_get_integer_v = unsafe {
            mem::transmute::<
                _,
                unsafe extern "C" fn(pname: gl::types::GLenum, data: *mut gl::types::GLint),
            >(gl_display.get_proc_address(cstr!("glGetIntegerv")) as *const ())
        };

        state.graphics = GraphicsState::OpenGL {
            window: window,
            display: gl_display,
            context: current_context,
            surface: surface,
            gl_get_integer_v: gl_get_integer_v,
        };
        Ok(())
    } else {
        Err(M64PError::InvalidState)
    }
}

fn gen_config_builder(
    bits_per_pixel: c_int,
    attr_map: &mut HashMap<GLAttribute, c_int>,
) -> ConfigTemplateBuilder {
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
        builder = builder.with_single_buffering(is_single_buffer);
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

    builder
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
