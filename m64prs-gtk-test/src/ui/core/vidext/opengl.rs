use std::{
    ffi::{c_int, c_void, CStr, CString},
    num::NonZero,
};

use glutin::{
    config::{Api, ColorBufferType, ConfigTemplateBuilder, GetGlConfig, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version},
    display::{Display, DisplayApiPreference},
    prelude::{GlDisplay, NotCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use m64prs_core::{error::M64PError, vidext::VidextResult};
use m64prs_sys::{GLAttribute, GLContextType};
use num_enum::TryFromPrimitive;
use raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
};

use crate::{
    controls::SubsurfaceHandle,
    ui::gl::{
        self,
        types::{GLenum, GLint},
        Gl,
    },
};

pub(super) enum OpenGlState {
    Config(OpenGlConfigState),
    Active(OpenGlActiveState),
}

impl Default for OpenGlState {
    fn default() -> Self {
        OpenGlState::Config(OpenGlConfigState::default())
    }
}

pub(super) struct OpenGlConfigState {
    red_size: u8,
    green_size: u8,
    blue_size: u8,
    alpha_size: u8,
    depth_size: u8,
    multisampling: u8,
    context_major_version: u8,
    context_minor_version: u8,
    context_profile_mask: GLContextType,
}

pub(super) struct OpenGlActiveState {
    subsurface: SubsurfaceHandle,

    gl_display: Display,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: Gl,
}

impl Default for OpenGlConfigState {
    fn default() -> Self {
        Self {
            red_size: 8,
            green_size: 8,
            blue_size: 8,
            alpha_size: 0,
            depth_size: 0,
            multisampling: 0,
            context_major_version: 3,
            context_minor_version: 3,
            context_profile_mask: GLContextType::Compatibility,
        }
    }
}

impl OpenGlConfigState {
    pub(super) fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> VidextResult<()> {
        fn into_u8(value: c_int) -> VidextResult<u8> {
            value.try_into().map_err(|_| M64PError::InputAssert)
        }

        match attr {
            GLAttribute::Doublebuffer => Ok(()),
            GLAttribute::BufferSize => Ok(()),
            GLAttribute::DepthSize => {
                self.depth_size = into_u8(value)?;
                Ok(())
            }
            GLAttribute::RedSize => {
                self.red_size = into_u8(value)?;
                Ok(())
            }
            GLAttribute::GreenSize => {
                self.green_size = into_u8(value)?;
                Ok(())
            }
            GLAttribute::BlueSize => {
                self.blue_size = into_u8(value)?;
                Ok(())
            }
            GLAttribute::AlphaSize => {
                self.alpha_size = into_u8(value)?;
                Ok(())
            }
            GLAttribute::SwapControl => Ok(()),
            GLAttribute::Multisamplebuffers => Ok(()),
            GLAttribute::Multisamplesamples => {
                let value = into_u8(value)?;
                if (value & (value - 1)) == 0u8 {
                    self.multisampling = value;
                    Ok(())
                } else {
                    Err(M64PError::InputAssert)
                }
            }
            GLAttribute::ContextMajorVersion => {
                self.context_major_version = into_u8(value)?;
                Ok(())
            }
            GLAttribute::ContextMinorVersion => {
                self.context_minor_version = into_u8(value)?;
                Ok(())
            }
            GLAttribute::ContextProfileMask => {
                let profile = GLContextType::try_from(
                    value as <GLContextType as TryFromPrimitive>::Primitive,
                )
                .map_err(|_| M64PError::InputAssert)?;
                if profile == GLContextType::Es {
                    return Err(M64PError::Unsupported);
                }
                Ok(())
            }
        }
    }

    pub(super) fn window_request_params(
        &self,
        width: c_int,
        height: c_int,
    ) -> (graphene::Point, dpi::PhysicalSize<u32>, bool) {
        let size = dpi::PhysicalSize::new(width, height).cast::<u32>();
        (graphene::Point::zero(), size, self.alpha_size > 0)
    }

    pub(super) fn setup_opengl_context(
        self,
        subsurface: SubsurfaceHandle,
        size: dpi::PhysicalSize<u32>,
    ) -> Result<OpenGlActiveState, (M64PError, OpenGlConfigState)> {
        // We can't just use the question mark operator, so we do this.
        macro_rules! check {
            ($exp:expr, $err:expr $(,)?) => {
                match $exp {
                    Ok(value) => value,
                    Err(_) => return Err(($err, self)),
                }
            };
            ($exp:expr, $err:expr, $fmt:literal $(, $($args:expr),* $(,)?)?) => {
                match $exp {
                    Ok(value) => value,
                    Err(_) => {
                        ::log::error!($fmt, $(, $($args),*)?);
                        return Err(($err, self));
                    },
                }
            };
            ($exp:expr, $err:expr, [$errp:ident] $fmt:literal $(, $($args:expr),* $(,)?)?) => {
                match $exp {
                    Ok(value) => value,
                    Err($errp) => {
                        ::log::error!($fmt $(, $($args),*)?);
                        return Err(($err, self));
                    },
                }
            };
        }

        let display_handle = check!(subsurface.display_handle(), M64PError::SystemFail);
        let window_handle = check!(subsurface.window_handle(), M64PError::SystemFail);

        let gl_display = {
            let api_preference = match display_handle.as_raw() {
                RawDisplayHandle::Wayland(_) => DisplayApiPreference::Egl,
                RawDisplayHandle::Xcb(_) => DisplayApiPreference::Egl,
                _ => unimplemented!(),
            };

            check!(
                unsafe { Display::new(display_handle.as_raw(), api_preference) },
                M64PError::SystemFail,
                "glutin Display::new should succeed"
            )
        };

        log::debug!("Created GL display");

        let gl_config = {
            let mut builder = ConfigTemplateBuilder::new()
                .with_api(match self.context_profile_mask {
                    GLContextType::Core | GLContextType::Compatibility => Api::OPENGL,
                    GLContextType::Es => match self.context_major_version {
                        1 => Api::GLES1,
                        2 => Api::GLES2,
                        3 => Api::GLES3,
                        _ => panic!("Invalid GLES major version"),
                    },
                })
                .with_buffer_type(ColorBufferType::Rgb {
                    r_size: self.red_size,
                    g_size: self.green_size,
                    b_size: self.blue_size,
                })
                .with_alpha_size(self.alpha_size)
                .with_depth_size(self.depth_size);
            if self.multisampling > 0 {
                builder = builder.with_multisampling(self.multisampling);
            }
            builder = builder.compatible_with_native_window(window_handle.as_raw());

            let result = check!(
                unsafe { gl_display.find_configs(builder.build()) },
                M64PError::SystemFail
            )
            .next();

            result.expect("No valid OpenGL configs available")
        };

        log::debug!("Found GL config");

        let gl_surface = {
            // safety: it was previously asserted that size is non-zero in either dimension.
            let nz_width = unsafe { NonZero::new_unchecked(size.width) };
            let nz_height = unsafe { NonZero::new_unchecked(size.height) };

            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
                .build(window_handle.as_raw(), nz_width, nz_height);

            check!(
                unsafe { gl_display.create_window_surface(&gl_config, &attrs) },
                M64PError::SystemFail,
                [err] "Failed to create window surface: {}", err
            )
        };

        log::debug!("Created GL surface");

        let gl_context = {
            let attrs = match self.context_profile_mask {
                GLContextType::Core => ContextAttributesBuilder::new()
                    .with_context_api(ContextApi::OpenGl(Some(Version::new(
                        self.context_major_version,
                        self.context_minor_version,
                    ))))
                    .with_profile(GlProfile::Core),
                GLContextType::Compatibility => ContextAttributesBuilder::new()
                    .with_context_api(ContextApi::OpenGl(Some(Version::new(
                        self.context_major_version,
                        self.context_minor_version,
                    ))))
                    .with_profile(GlProfile::Compatibility),
                GLContextType::Es => {
                    ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(Some(
                        Version::new(self.context_major_version, self.context_minor_version),
                    )))
                }
            }
            .build(Some(window_handle.as_raw()));

            let context = check!(
                unsafe { gl_display.create_context(&gl_config, &attrs) },
                M64PError::SystemFail
            );
            check!(context.make_current(&gl_surface), M64PError::SystemFail)
        };

        log::debug!("Created GL context");

        let gl = Gl::load_with(|sym| gl_display.get_proc_address(&CString::new(sym).unwrap()));

        log::debug!("Loaded GL functions");

        Ok(OpenGlActiveState {
            subsurface,
            gl_display,
            gl_context,
            gl_surface,
            gl,
        })
    }
}

impl OpenGlActiveState {
    pub(super) fn swap_buffers(&mut self) -> VidextResult<()> {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .map_err(|_| M64PError::SystemFail)
    }

    pub(super) fn gl_get_attribute(&mut self, attr: GLAttribute) -> VidextResult<c_int> {
        let config = self.gl_context.config();

        match attr {
            GLAttribute::Doublebuffer => Ok(1),
            GLAttribute::BufferSize => {
                let color_size = match config.color_buffer_type() {
                    Some(ColorBufferType::Luminance(y_size)) => y_size,
                    Some(ColorBufferType::Rgb {
                        r_size,
                        g_size,
                        b_size,
                    }) => r_size + g_size + b_size,
                    None => 0,
                } as c_int;
                let alpha_size = config.alpha_size() as c_int;
                Ok(color_size + alpha_size)
            }
            GLAttribute::DepthSize => Ok(config.depth_size() as c_int),
            GLAttribute::RedSize => match config.color_buffer_type() {
                Some(ColorBufferType::Rgb { r_size, .. }) => Ok(r_size as c_int),
                _ => Ok(0),
            },
            GLAttribute::GreenSize => match config.color_buffer_type() {
                Some(ColorBufferType::Rgb { g_size, .. }) => Ok(g_size as c_int),
                _ => Ok(0),
            },
            GLAttribute::BlueSize => match config.color_buffer_type() {
                Some(ColorBufferType::Rgb { b_size, .. }) => Ok(b_size as c_int),
                _ => Ok(0),
            },
            GLAttribute::AlphaSize => Ok(config.alpha_size() as c_int),
            GLAttribute::SwapControl => Err(M64PError::Unsupported),
            GLAttribute::Multisamplebuffers => Ok((config.num_samples() > 0) as c_int),
            GLAttribute::Multisamplesamples => Ok(config.num_samples() as c_int),
            GLAttribute::ContextMajorVersion => unsafe {
                let mut version: GLint = 0;
                self.gl.GetIntegerv(gl::MAJOR_VERSION, &mut version);
                Ok(version as c_int)
            },
            GLAttribute::ContextMinorVersion => unsafe {
                let mut version: GLint = 0;
                self.gl.GetIntegerv(gl::MINOR_VERSION, &mut version);
                Ok(version as c_int)
            },
            GLAttribute::ContextProfileMask => unsafe {
                let mut mask: GLint = 0;
                self.gl.GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut mask);
                if ((mask as GLenum) & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT) != 0 {
                    Ok(GLContextType::Compatibility as c_int)
                } else {
                    Ok(GLContextType::Core as c_int)
                }
            },
        }
    }

    pub(super) fn get_proc_address(&mut self, sym: &CStr) -> *mut c_void {
        self.gl_display.get_proc_address(sym) as *mut c_void
    }
}
