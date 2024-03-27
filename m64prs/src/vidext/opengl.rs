use std::{
    ffi::{c_int, c_uint, c_void, CStr, CString},
    num::NonZeroU32,
    time::Duration,
};

use gl::types::{GLenum, GLint};
use glutin::{
    config::{ColorBufferType, ConfigTemplateBuilder, GetGlConfig, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, GlContext, GlProfile, NotCurrentGlContext,
        PossiblyCurrentContext, PossiblyCurrentGlContext, Version,
    },
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::GlWindow;
use log::warn;
use m64prs_core::{error::M64PError, types::FFIResult};
use m64prs_sys::{GLAttribute, GLContextType, VideoFlags, VideoMode};
use rwh_05::HasRawWindowHandle;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    platform::pump_events::EventLoopExtPumpEvents,
    window::{Window, WindowBuilder},
};

use super::EVENT_LOOP;

pub(super) struct OpenGlInitState {
    color_buffer_type: ColorBufferType,
    alpha_size: u8,
    depth_size: u8,
    swap_control: u16,
    msaa_samples: u8,
    gl_version_major: u8,
    gl_version_minor: u8,
    context_type: GLContextType,
}

impl Default for OpenGlInitState {
    fn default() -> Self {
        Self {
            color_buffer_type: ColorBufferType::Rgb {
                r_size: 8u8,
                g_size: 8u8,
                b_size: 8u8,
            },
            alpha_size: 8u8,
            depth_size: 24u8,
            swap_control: 0u16,
            msaa_samples: 1u8,
            gl_version_major: 3u8,
            gl_version_minor: 3u8,
            context_type: GLContextType::Compatibility,
        }
    }
}

impl OpenGlInitState {
    pub(crate) fn set_attr(&mut self, attr: GLAttribute, value: c_int) -> FFIResult<()> {
        match attr {
            GLAttribute::Doublebuffer => {
                if value == 0 {
                    Err(M64PError::InputInvalid)
                } else {
                    Ok(())
                }
            }
            GLAttribute::BufferSize => {
                match value {
                    32 => {
                        self.color_buffer_type = ColorBufferType::Rgb {
                            r_size: 8u8,
                            g_size: 8u8,
                            b_size: 8u8,
                        };
                        self.alpha_size = 8u8;
                    }
                    24 => {
                        self.color_buffer_type = ColorBufferType::Rgb {
                            r_size: 8u8,
                            g_size: 8u8,
                            b_size: 8u8,
                        };
                        self.alpha_size = 0u8;
                    }
                    _ => return Err(M64PError::InputAssert),
                }
                Ok(())
            }
            GLAttribute::DepthSize => {
                self.depth_size = value.try_into().map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
            GLAttribute::RedSize => {
                if let ColorBufferType::Rgb { r_size, .. } = &mut self.color_buffer_type {
                    *r_size = value.try_into().map_err(|_| M64PError::InputAssert)?;
                }
                Ok(())
            }
            GLAttribute::GreenSize => {
                if let ColorBufferType::Rgb { g_size, .. } = &mut self.color_buffer_type {
                    *g_size = value.try_into().map_err(|_| M64PError::InputAssert)?;
                }
                Ok(())
            }
            GLAttribute::BlueSize => {
                if let ColorBufferType::Rgb { b_size, .. } = &mut self.color_buffer_type {
                    *b_size = value.try_into().map_err(|_| M64PError::InputAssert)?;
                }
                Ok(())
            }
            GLAttribute::AlphaSize => {
                self.alpha_size = value.try_into().map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
            GLAttribute::SwapControl => {
                self.swap_control = value.try_into().map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
            GLAttribute::Multisamplebuffers => Ok(()),
            GLAttribute::Multisamplesamples => {
                self.msaa_samples = value.try_into().map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
            GLAttribute::ContextMajorVersion => {
                self.gl_version_major = value.try_into().map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
            GLAttribute::ContextMinorVersion => {
                self.gl_version_minor = value.try_into().map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
            GLAttribute::ContextProfileMask => {
                // You can't just do c_int -> GLContextType, so we do c_int -> c_uint -> GLContextType
                self.context_type = c_uint::try_from(value)
                    .map_err(|_| M64PError::InputAssert)?
                    .try_into()
                    .map_err(|_| M64PError::InputAssert)?;
                Ok(())
            }
        }
    }

    pub(crate) fn generate_config_template(&self) -> ConfigTemplateBuilder {
        let mut builder = ConfigTemplateBuilder::new()
            .with_buffer_type(self.color_buffer_type)
            .with_alpha_size(self.alpha_size)
            .with_depth_size(self.depth_size)
            .with_multisampling(self.msaa_samples);

        builder = match self.swap_control {
            0u16 => builder.with_swap_interval(None, None),
            value => builder.with_swap_interval(Some(value), None),
        };

        builder
    }

    pub(crate) fn generate_context_attributes(&self) -> ContextAttributesBuilder {
        let builder = ContextAttributesBuilder::new();

        match self.context_type {
            GLContextType::Core => builder
                .with_context_api(ContextApi::OpenGl(Some(Version {
                    major: self.gl_version_major,
                    minor: self.gl_version_minor,
                })))
                .with_profile(GlProfile::Core),
            GLContextType::Compatibility => builder
                .with_context_api(ContextApi::OpenGl(Some(Version {
                    major: self.gl_version_major,
                    minor: self.gl_version_minor,
                })))
                .with_profile(GlProfile::Compatibility),
            GLContextType::Es => builder.with_context_api(ContextApi::Gles(Some(Version {
                major: self.gl_version_major,
                minor: self.gl_version_minor,
            }))),
        }
    }

    pub(crate) fn generate_surface_attributes(&self) -> SurfaceAttributesBuilder<WindowSurface> {
        let builder = SurfaceAttributesBuilder::<WindowSurface>::new();
        builder
    }
}

pub(crate) struct OpenGlInitParams {
    pub width: c_int,
    pub height: c_int,
    pub bits_per_pixel: c_int,
    pub screen_mode: VideoMode,
    pub video_flags: VideoFlags,
}

pub(crate) struct OpenGlActiveState {
    window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    swap_interval: SwapInterval,
}

impl OpenGlActiveState {
    pub(crate) fn init_from(state: &OpenGlInitState, params: OpenGlInitParams) -> FFIResult<Self> {
        if params.screen_mode != VideoMode::Windowed {
            return Err(M64PError::Unsupported);
        }

        let event_loop = EVENT_LOOP.get().ok_or(M64PError::Internal)?;
        event_loop
            .with(|event_loop| -> FFIResult<OpenGlActiveState> {
                let window_builder = WindowBuilder::new()
                    .with_transparent(true)
                    .with_inner_size(LogicalSize::new(params.width, params.height));

                let template_builder = state.generate_config_template();
                let (window, gl_config) = glutin_winit::DisplayBuilder::new()
                    .with_window_builder(Some(window_builder))
                    .build(&event_loop, template_builder, |mut iter| {
                        iter.next().unwrap()
                    })
                    .map_err(|_| M64PError::SystemFail)?;

                let window = window.ok_or(M64PError::SystemFail)?;
                window.set_cursor_visible(false);
                let window_handle = window.raw_window_handle();

                let gl_display = gl_config.display();
                let gl_context = unsafe {
                    gl_display
                        .create_context(
                            &gl_config,
                            &state
                                .generate_context_attributes()
                                .build(Some(window_handle)),
                        )
                        .map_err(|_| M64PError::SystemFail)?
                };
                let gl_surface = unsafe {
                    gl_display
                        .create_window_surface(
                            &gl_config,
                            &window.build_surface_attributes(state.generate_surface_attributes()),
                        )
                        .map_err(|_| M64PError::SystemFail)?
                };
                let gl_context = gl_context
                    .make_current(&gl_surface)
                    .map_err(|_| M64PError::SystemFail)?;

                let swap_interval = match state.swap_control {
                    0 => SwapInterval::DontWait,
                    1.. => SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
                };
                if let Err(_) = gl_surface.set_swap_interval(&gl_context, swap_interval) {
                    warn!("Failed to set swap interval. Plugin may not behave as intended.");
                }

                // This is necessary to distinguish core from compatibility contexts.
                {
                    let load_fn = |ptr: &'static str| {
                        gl_display.get_proc_address(&CString::new(ptr).unwrap())
                    };
                    gl::GetIntegerv::load_with(load_fn);
                }

                Ok(OpenGlActiveState {
                    window: window,
                    surface: gl_surface,
                    context: gl_context,
                    swap_interval,
                })
            })
            .map_err(|_| M64PError::Internal)?
    }

    pub(crate) fn swap_buffers(&mut self) -> FFIResult<()> {
        self.window.request_redraw();

        let id = self.window.id();

        EVENT_LOOP
            .get()
            .unwrap()
            .with(|event_loop| {
                event_loop.pump_events(Some(Duration::ZERO), |event, _| {
                    // println!("{:?}", event);
                    match event {
                        Event::WindowEvent { window_id, event } => match event {
                            WindowEvent::Resized(size) => {
                                if size.width != 0 && size.height != 0 {
                                    self.surface.resize(
                                        &self.context,
                                        NonZeroU32::new(size.width).unwrap(),
                                        NonZeroU32::new(size.height).unwrap(),
                                    );
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                if id == window_id {
                                    self.surface.swap_buffers(&self.context).unwrap();
                                }
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                });
            })
            .map_err(|_| M64PError::Internal)?;

        Ok(())
    }

    pub(crate) fn get_proc_address(&self, symbol: &CStr) -> *mut c_void {
        self.context.display().get_proc_address(symbol) as *mut c_void
    }

    pub(crate) fn get_attr(&self, attr: GLAttribute) -> FFIResult<c_int> {
        match attr {
            GLAttribute::Doublebuffer => Ok(1),
            GLAttribute::BufferSize => {
                let config = self.context.config();
                let color_size = match config.color_buffer_type() {
                    Some(ColorBufferType::Rgb {
                        r_size,
                        g_size,
                        b_size,
                    }) => r_size + g_size + b_size,
                    None => 0,
                    _ => return Err(M64PError::Internal),
                };
                Ok(config.alpha_size() as c_int + color_size as c_int)
            }
            GLAttribute::DepthSize => Ok(self.context.config().depth_size() as c_int),
            GLAttribute::RedSize => match self.context.config().color_buffer_type() {
                Some(ColorBufferType::Rgb { r_size, .. }) => Ok(r_size as c_int),
                None => Err(M64PError::InputNotFound),
                _ => return Err(M64PError::Internal),
            },
            GLAttribute::GreenSize => match self.context.config().color_buffer_type() {
                Some(ColorBufferType::Rgb { g_size, .. }) => Ok(g_size as c_int),
                None => Err(M64PError::InputNotFound),
                _ => return Err(M64PError::Internal),
            },
            GLAttribute::BlueSize => match self.context.config().color_buffer_type() {
                Some(ColorBufferType::Rgb { b_size, .. }) => Ok(b_size as c_int),
                None => Err(M64PError::InputNotFound),
                _ => return Err(M64PError::Internal),
            },
            GLAttribute::AlphaSize => Ok(self.context.config().alpha_size() as c_int),
            GLAttribute::SwapControl => match self.swap_interval {
                SwapInterval::DontWait => Ok(0),
                SwapInterval::Wait(n) => Ok(u32::from(n) as c_int),
            },
            GLAttribute::Multisamplebuffers => {
                if self.context.config().num_samples() > 1 {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            GLAttribute::Multisamplesamples => Ok(self.context.config().num_samples() as c_int),
            GLAttribute::ContextMajorVersion => match self.context.context_api() {
                ContextApi::OpenGl(Some(version)) => Ok(version.major as c_int),
                ContextApi::Gles(Some(version)) => Ok(version.major as c_int),
                _ => Err(M64PError::SystemFail),
            },
            GLAttribute::ContextMinorVersion => match self.context.context_api() {
                ContextApi::OpenGl(Some(version)) => Ok(version.minor as c_int),
                ContextApi::Gles(Some(version)) => Ok(version.minor as c_int),
                _ => Err(M64PError::SystemFail),
            },
            GLAttribute::ContextProfileMask => match self.context.context_api() {
                ContextApi::OpenGl(Some(version)) => {
                    let display = self.surface.display();
                    if version.major < 3 || version.major == 3 && version.minor < 1 {
                        Ok(u32::from(GLContextType::Core) as c_int)
                    } else {
                        let mut profile_mask: GLint = 0;
                        unsafe {
                            gl::GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut profile_mask);
                        }

                        if profile_mask as GLenum & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT != 0 {
                            Ok(u32::from(GLContextType::Compatibility) as c_int)
                        } else {
                            Ok(u32::from(GLContextType::Core) as c_int)
                        }
                    }
                }
                ContextApi::Gles(_) => Ok(u32::from(GLContextType::Es) as c_int),
                _ => Err(M64PError::Internal),
            },
        }
    }
}
