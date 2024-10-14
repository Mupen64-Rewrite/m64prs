use std::{
    ffi::{c_int, c_void, CStr, CString},
    mem,
    num::NonZeroU32,
    ptr::null_mut,
    sync::Arc,
    time::Duration,
};

mod gl {
    include!(concat!(env!("OUT_DIR"), "/opengl.gen.rs"));
}

use gl::{
    types::{GLenum, GLint},
    Gl,
};
use glutin::{
    config::{Api, ColorBufferType, Config, ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version},
    display::{Display, GetGlDisplay},
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use m64prs_core::{
    error::M64PError,
    types::FFIResult,
    Core,
};
use m64prs_sys::{GLAttribute, GLContextType, VideoFlags, VideoMode};
use raw_window_handle::HasWindowHandle;
use std::sync::RwLock;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
    window::Window,
};

use super::VideoUserEvent;

pub enum OpenGlEvent {}

#[derive(Debug)]
pub struct OpenGlInitState {
    double_buffer: bool,
    depth_bits: u8,
    red_bits: u8,
    green_bits: u8,
    blue_bits: u8,
    alpha_bits: u8,
    swap_control: u16,
    multisample_samples: u8,
    gl_major_version: u8,
    gl_minor_version: u8,
    gl_context_type: GLContextType,
}

impl Default for OpenGlInitState {
    fn default() -> Self {
        Self {
            double_buffer: true,
            depth_bits: 24,
            red_bits: 8,
            green_bits: 8,
            blue_bits: 8,
            alpha_bits: 0,
            swap_control: 1,
            multisample_samples: 0,
            gl_major_version: 3,
            gl_minor_version: 3,
            gl_context_type: GLContextType::Compatibility,
        }
    }
}

impl OpenGlInitState {
    fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> Result<(), M64PError> {
        match attr {
            GLAttribute::Doublebuffer => self.double_buffer = value != 0,
            GLAttribute::BufferSize => (),
            // match value {
            //     32 => {
            //         self.red_bits = 8;
            //         self.green_bits = 8;
            //         self.blue_bits = 8;
            //         self.alpha_bits = 8;
            //     }
            //     24 => {
            //         self.red_bits = 8;
            //         self.green_bits = 8;
            //         self.blue_bits = 8;
            //         self.alpha_bits = 0;
            //     }
            //     _ => return Err(M64PError::InputAssert),
            // },
            GLAttribute::DepthSize => {
                self.depth_bits = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::RedSize => {
                self.red_bits = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::GreenSize => {
                self.green_bits = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::BlueSize => {
                self.blue_bits = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::AlphaSize => {
                self.alpha_bits = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::SwapControl => {
                self.swap_control = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::Multisamplebuffers => (),
            GLAttribute::Multisamplesamples => {
                if (value & (value - 1)) != 0 {
                    return Err(M64PError::InputAssert);
                }
                self.multisample_samples = value.try_into().map_err(|_| M64PError::InputAssert)?;
            }
            GLAttribute::ContextMajorVersion => {
                self.gl_major_version = value.try_into().map_err(|_| M64PError::InputAssert)?
            }
            GLAttribute::ContextMinorVersion => {
                self.gl_minor_version = value.try_into().map_err(|_| M64PError::InputAssert)?
            }
            GLAttribute::ContextProfileMask => {
                self.gl_context_type = (value as u32)
                    .try_into()
                    .map_err(|_| M64PError::InputAssert)?
            }
        }

        Ok(())
    }

    fn ready(
        self,
        core: Arc<RwLock<Core>>,
        width: NonZeroU32,
        height: NonZeroU32,
        video_mode: VideoMode,
        video_flags: VideoFlags,
    ) -> Result<OpenGlReadyState, M64PError> {
        Ok(OpenGlReadyState {
            core,
            init_state: self,
            width,
            height,
            video_mode,
            video_flags,
        })
    }
}

pub struct OpenGlReadyState {
    init_state: OpenGlInitState,
    core: Arc<RwLock<Core>>,
    width: NonZeroU32,
    height: NonZeroU32,
    video_mode: VideoMode,
    video_flags: VideoFlags,
}

impl OpenGlReadyState {
    fn activate(
        self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<OpenGlActiveState, M64PError> {
        log::trace!("Creating window attributes");
        // setup window and OpenGL config attributes
        let mut window_attrs = Window::default_attributes()
            .with_title("m64prs")
            .with_inner_size(PhysicalSize::<u32>::new(
                self.width.into(),
                self.height.into(),
            ))
            .with_transparent(self.init_state.alpha_bits > 0);
        if !self.video_flags.contains(VideoFlags::SUPPORT_RESIZING) {
            window_attrs = window_attrs.with_resizable(false);
        }

        log::trace!("Creating OpenGL configuration");

        let mut template = ConfigTemplateBuilder::new()
            .with_buffer_type(glutin::config::ColorBufferType::Rgb {
                r_size: self.init_state.red_bits,
                g_size: self.init_state.green_bits,
                b_size: self.init_state.blue_bits,
            })
            .with_alpha_size(self.init_state.alpha_bits)
            .with_depth_size(self.init_state.depth_bits)
            .with_stencil_size(8)
            .with_swap_interval(
                None,
                if self.init_state.swap_control == 0 {
                    None
                } else {
                    Some(self.init_state.swap_control)
                },
            )
            .with_single_buffering(!self.init_state.double_buffer);

        if self.init_state.multisample_samples > 0 {
            template = template.with_multisampling(self.init_state.multisample_samples)
        }

        // build window and OpenGL config
        log::debug!("Setting up window with OpenGL configuration");
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attrs))
            .build(event_loop, template, |mut configs| {
                configs.next().expect("no configs? that's balls")
            })
            .map(|(window, gl_config)| (window.unwrap(), gl_config))
            .map_err(|_| M64PError::SystemFail)?;

        // acquire handles to init other OpenGL objects
        log::trace!("Acquiring window and display handles");
        let window_handle = window
            .window_handle()
            .map_err(|_| M64PError::SystemFail)?
            .as_raw();
        let gl_display = gl_config.display();

        // setup OpenGL surface
        log::debug!("Creating OpenGL surface attributes");
        let surface_attrs = window
            .build_surface_attributes(
                SurfaceAttributesBuilder::<WindowSurface>::new()
                    .with_single_buffer(!self.init_state.double_buffer),
            )
            .map_err(|_| M64PError::SystemFail)?;

        log::debug!("Setting up OpenGL surface");
        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attrs) }
            .map_err(|_| M64PError::SystemFail)?;

        log::trace!("Creating OpenGL context attributes");
        // setup OpenGL context
        let context_attrs = match (
            self.init_state.gl_context_type,
            self.init_state.gl_major_version,
        ) {
            // For OpenGL < 3.0, profile doesn't make sense
            (GLContextType::Core, ..=2) | (GLContextType::Compatibility, ..=2) => {
                ContextAttributesBuilder::new().with_context_api(ContextApi::OpenGl(Some(
                    Version {
                        major: self.init_state.gl_major_version,
                        minor: self.init_state.gl_minor_version,
                    },
                )))
            }
            // For OpenGL >= 3.0, it does
            (GLContextType::Core, 3..) => ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version {
                    major: self.init_state.gl_major_version,
                    minor: self.init_state.gl_minor_version,
                })))
                .with_profile(GlProfile::Core),
            (GLContextType::Compatibility, 3..) => ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version {
                    major: self.init_state.gl_major_version,
                    minor: self.init_state.gl_minor_version,
                })))
                .with_profile(GlProfile::Compatibility),
            // OpenGL ES
            (GLContextType::Es, _) => {
                ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(Some(Version {
                    major: self.init_state.gl_major_version,
                    minor: self.init_state.gl_minor_version,
                })))
            }
        }
        .build(Some(window_handle));

        log::debug!("Creating OpenGL context");
        // create OpenGL context and make it current
        let gl_context = unsafe { gl_display.create_context(&gl_config, &context_attrs) }
            .and_then(|context| context.make_current(&gl_surface))
            .map_err(|_| M64PError::SystemFail)?;

        log::debug!("Loading OpenGL functions");
        let gl = gl::Gl::load_with(|s| {
            gl_display
                .get_proc_address(&CString::new(s).expect("invalid symbol found during loading"))
        });

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::DontWait)
            .map_err(|_| M64PError::SystemFail)?;

        log::info!("OpenGL successfully started");
        // transfer all objects we created to active state, also the core
        Ok(OpenGlActiveState {
            core: self.core,
            window,
            // saved parameters
            video_flags: self.video_flags,
            // OpenGL objects
            gl,
            gl_display,
            gl_config,
            gl_context,
            gl_surface,
        })
    }
}

pub struct OpenGlActiveState {
    core: Arc<RwLock<Core>>,
    window: Window,
    // saved parameters
    video_flags: VideoFlags,
    // OpenGL objects
    gl: Gl,
    gl_display: Display,
    gl_config: Config,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
}

impl OpenGlActiveState {
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) -> Result<(), M64PError> {
        if window_id != self.window.id() {
            return Ok(());
        }
        match event {
            WindowEvent::CloseRequested => {
                let core = self.core.read().unwrap();
                core.stop().map_err(|_| M64PError::PluginFail)?;
            }
            WindowEvent::RedrawRequested => {
                self.window.pre_present_notify();
                self.gl_surface
                    .swap_buffers(&self.gl_context)
                    .map_err(|_| M64PError::SystemFail)?;
            }
            WindowEvent::Resized(size) => {
                if self.video_flags.contains(VideoFlags::SUPPORT_RESIZING) {
                    let core = self.core.read().unwrap();
                    match core.notify_resize(
                        size.width.try_into().map_err(|_| M64PError::Internal)?,
                        size.height.try_into().map_err(|_| M64PError::Internal)?,
                    ) {
                        Ok(_) => (),
                        Err(M64PError::InvalidState) => (),
                        Err(_) => return Err(M64PError::Internal),
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn request_redraw(&mut self) -> FFIResult<()> {
        self.window.request_redraw();
        Ok(())
    }

    fn gl_get_attribute(&self, attr: GLAttribute) -> Result<c_int, M64PError> {
        match attr {
            GLAttribute::Doublebuffer => match self.gl_surface.is_single_buffered() {
                true => Ok(0),
                false => Ok(1),
            },
            GLAttribute::BufferSize => {
                let color_size = match self.gl_config.color_buffer_type() {
                    Some(ColorBufferType::Rgb {
                        r_size,
                        g_size,
                        b_size,
                    }) => r_size + g_size + b_size,
                    Some(ColorBufferType::Luminance(y_size)) => y_size,
                    None => todo!(),
                };
                Ok((color_size + self.gl_config.alpha_size()).into())
            }
            GLAttribute::DepthSize => Ok(self.gl_config.depth_size().into()),
            GLAttribute::RedSize => match self.gl_config.color_buffer_type() {
                Some(ColorBufferType::Rgb { r_size, .. }) => Ok(r_size.into()),
                _ => Err(M64PError::InputInvalid),
            },
            GLAttribute::GreenSize => match self.gl_config.color_buffer_type() {
                Some(ColorBufferType::Rgb { g_size, .. }) => Ok(g_size.into()),
                _ => Err(M64PError::InputInvalid),
            },
            GLAttribute::BlueSize => match self.gl_config.color_buffer_type() {
                Some(ColorBufferType::Rgb { b_size, .. }) => Ok(b_size.into()),
                _ => Err(M64PError::InputInvalid),
            },
            GLAttribute::AlphaSize => Ok(self.gl_config.alpha_size().into()),
            GLAttribute::SwapControl => Err(M64PError::Unsupported),
            GLAttribute::Multisamplebuffers => match self.gl_config.num_samples() {
                1.. => Ok(1),
                0 => Ok(0),
            },
            GLAttribute::Multisamplesamples => Ok(self.gl_config.num_samples().into()),
            GLAttribute::ContextMajorVersion => unsafe {
                let mut version: GLint = 0;
                self.gl
                    .GetIntegerv(gl::MAJOR_VERSION, &mut version as *mut GLint);
                Ok(version as c_int)
            },
            GLAttribute::ContextMinorVersion => unsafe {
                let mut version: GLint = 0;
                self.gl
                    .GetIntegerv(gl::MINOR_VERSION, &mut version as *mut GLint);
                Ok(version as c_int)
            },
            GLAttribute::ContextProfileMask => {
                if self
                    .gl_config
                    .api()
                    .intersects(Api::GLES1 | Api::GLES2 | Api::GLES3)
                {
                    Ok(GLContextType::Es as u32 as i32)
                } else if self.gl_config.api().intersects(Api::OPENGL) {
                    unsafe {
                        let mut version: GLint = 0;
                        self.gl
                            .GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut version as *mut GLint);
                        if ((version as GLenum) & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT) != 0 {
                            Ok(GLContextType::Compatibility as u32 as i32)
                        } else {
                            Ok(GLContextType::Core as u32 as i32)
                        }
                    }
                } else {
                    Err(M64PError::Internal)
                }
            }
        }
    }

    fn gl_get_proc_address(&self, symbol: &CStr) -> *mut c_void {
        self.gl_display.get_proc_address(symbol) as *mut c_void
    }

    fn resize_window(&self, width: NonZeroU32, height: NonZeroU32) -> Result<(), M64PError> {
        self.gl_surface.resize(&self.gl_context, width, height);
        Ok(())
    }
}

pub enum OpenGlState {
    Empty,
    FatalError(M64PError),
    Init(OpenGlInitState),
    Ready(OpenGlReadyState),
    Active(OpenGlActiveState),
}

impl Default for OpenGlState {
    fn default() -> Self {
        Self::Empty
    }
}

impl ApplicationHandler<VideoUserEvent> for OpenGlState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match mem::take(self) {
            // ready state transitions to active
            Self::Ready(ready_state) => match ready_state.activate(event_loop) {
                Ok(active_state) => *self = Self::Active(active_state),
                Err(err) => *self = Self::FatalError(err),
            },
            // active state doesn't change
            Self::Active(active_state) => *self = Self::Active(active_state),
            Self::FatalError(_) => (),
            _ => *self = Self::FatalError(M64PError::Internal),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match self {
            Self::Active(active_state) => {
                if let Err(error) = active_state.window_event(event_loop, window_id, event) {
                    *self = Self::FatalError(error);
                }
            }
            Self::FatalError(_) => (),
            _ => *self = Self::FatalError(M64PError::Internal),
        }
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: VideoUserEvent,
    ) {
        match self {
            Self::Active(_active_state) => match event {
                VideoUserEvent::CoreQuitRequest => event_loop.exit(),
            },
            Self::FatalError(_) => (),
            _ => *self = Self::FatalError(M64PError::Internal),
        }
    }
}

impl OpenGlState {
    pub fn init() -> Self {
        log::info!("Setting up winit/OpenGL video extension");
        Self::Init(OpenGlInitState::default())
    }

    pub fn cleanup(&mut self, event_loop: &mut EventLoop<VideoUserEvent>) {
        // pump the event loop until it stops
        while let PumpStatus::Continue = event_loop.pump_app_events(None, self) {}
    }

    pub fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> FFIResult<()> {
        match self {
            Self::Init(init_state) => {
                log::debug!("Setting OpenGL attribute {:?} to {}", attr, value);
                init_state.gl_set_attribute(attr, value)
            }
            Self::FatalError(error) => Err(*error),
            _ => Err(M64PError::Internal),
        }
    }

    pub fn set_video_mode(
        &mut self,
        core: Arc<RwLock<Core>>,
        event_loop: &mut EventLoop<VideoUserEvent>,
        width: c_int,
        height: c_int,
        _bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()> {
        match self {
            Self::Init(init_state) => {
                log::info!(
                    "Initializing OpenGL window (size: {}x{}, can resize: {})",
                    width,
                    height,
                    flags.contains(VideoFlags::SUPPORT_RESIZING)
                );
                // Ensure width and height are positive integers
                let width = u32::try_from(width)
                    .and_then(|value| value.try_into())
                    .map_err(|_| M64PError::InputAssert)?;
                let height = u32::try_from(height)
                    .and_then(|value| value.try_into())
                    .map_err(|_| M64PError::InputAssert)?;

                // prepare to start the event loop
                match mem::take(init_state).ready(core, width, height, screen_mode, flags) {
                    Ok(ready_state) => {
                        *self = Self::Ready(ready_state);
                    }
                    Err(error) => {
                        *self = Self::FatalError(error);
                        return Err(error);
                    }
                }
                log::info!("Parameters ready. Starting event loop...");

                // start the event loop, transitioning to active
                // this is fallible, so if any errors occur, they are reported
                event_loop.pump_app_events(Some(Duration::ZERO), self);
                if let Self::FatalError(error) = self {
                    log::warn!("Error occurred during initial pump!");
                    return Err(*error);
                }
            }
            Self::FatalError(error) => return Err(*error),
            _ => return Err(M64PError::Internal),
        }
        Ok(())
    }

    pub fn gl_get_attribute(&mut self, attr: GLAttribute) -> FFIResult<c_int> {
        match self {
            Self::Active(active_state) => active_state.gl_get_attribute(attr),
            Self::FatalError(error) => Err(*error),
            _ => Err(M64PError::Internal),
        }
    }

    pub fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void {
        match self {
            Self::Active(active_state) => active_state.gl_get_proc_address(symbol),
            _ => null_mut(),
        }
    }

    pub fn gl_swap_buffers(&mut self, event_loop: &mut EventLoop<VideoUserEvent>) -> FFIResult<()> {
        match self {
            Self::Active(active_state) => {
                // Perform actions necessary to trigger buffer swap
                active_state.request_redraw()?;
                // swap buffers, poll event loop
                event_loop.pump_app_events(Some(Duration::ZERO), self);
            }
            Self::FatalError(error) => return Err(*error),
            _ => return Err(M64PError::Internal),
        }
        Ok(())
    }

    pub fn resize_window(&mut self, width: c_int, height: c_int) -> FFIResult<()> {
        match self {
            Self::Active(active_state) => active_state.resize_window(
                u32::try_from(width)
                    .and_then(|value| value.try_into())
                    .map_err(|_| M64PError::InputAssert)?,
                u32::try_from(height)
                    .and_then(|value| value.try_into())
                    .map_err(|_| M64PError::InputAssert)?,
            ),
            Self::FatalError(error) => Err(*error),
            _ => Err(M64PError::Internal),
        }
    }
}
