use std::{
    any::Any, ffi::{c_char, c_int, c_void, CStr}, fmt::Debug, ptr::null_mut, sync::mpsc
};

use ash::vk;
use graphene::{Point, Size};
use m64prs_core::{
    error::M64PError,
    vidext::{VideoExtension, VidextResult},
};
use m64prs_sys::RenderMode;
use opengl::OpenGlState;
use relm4::ComponentSender;
use request::RequestManager;

use crate::controls::SubsurfaceHandle;

mod opengl;
mod request;

#[derive(Debug)]
pub enum VidextRequest {
    EnterGameView,
    ExitGameView,
    CreateSubsurface {
        position: Point,
        size: dpi::Size,
        transparent: bool,
    },
}

#[derive(Debug)]
pub enum VidextResponse {
    Done,
    NewSubsurface(SubsurfaceHandle),
}

enum GraphicsState {
    Uninit,
    OpenGl(Option<OpenGlState>),
    // Vulkan
}

pub struct VideoExtensionParameters {
    outbound: ComponentSender<super::Model>,
    inbound: mpsc::Receiver<(usize, VidextResponse)>,
}

impl VideoExtensionParameters {
    /// Constructs the message queues.
    pub(super) fn new(
        sender: ComponentSender<super::Model>,
    ) -> (Self, mpsc::Sender<(usize, VidextResponse)>) {
        let (tx, rx) = mpsc::channel();

        let inst = Self {
            outbound: sender,
            inbound: rx,
        };
        (inst, tx)
    }
}

pub struct VideoExtensionState {
    request_mgr: RequestManager,
    graphics: GraphicsState,
}

impl Debug for VideoExtensionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoExtensionState").finish()
    }
}

impl VideoExtensionState {
    fn new(parameters: VideoExtensionParameters) -> Self {
        Self {
            request_mgr: RequestManager::new(parameters.outbound, parameters.inbound),
            graphics: GraphicsState::Uninit,
        }
    }
}

// TODO: implement vidext
#[allow(unused)]
impl VideoExtension for VideoExtensionState {
    unsafe fn init_with_render_mode(mode: RenderMode, context: &mut dyn Any) -> VidextResult<Self> {
        

        let context = context
            .downcast_mut::<Option<VideoExtensionParameters>>()
            .expect("expected Option<VideoExtensionParameters> from downcast");

        let parameters = context.take().unwrap();

        match mode {
            RenderMode::OpenGl => {
                let mut inst = Self::new(parameters);
                // Request the GUI to switch to the game view.
                log::info!("Requesting switch to game view");
                inst.request_mgr.request(VidextRequest::EnterGameView)
                    .map_err(|_| M64PError::Internal)?;

                log::info!("Init successful");
                inst.graphics = GraphicsState::OpenGl(Some(OpenGlState::default()));
                Ok(inst)
            }
            RenderMode::Vulkan => Err(M64PError::Unsupported),
        }
    }

    unsafe fn quit(self, context: &mut dyn Any) -> VidextResult<()> {
        let state = context
            .downcast_mut::<Option<VideoExtensionParameters>>()
            .expect("expected Option<VideoExtensionParameters> from downcast");

        let Self { graphics, request_mgr } = self;
        drop(graphics);

        let _ = request_mgr.request(VidextRequest::ExitGameView);
        *state = Some(request_mgr.cleanup());


        Ok(())
    }

    unsafe fn list_fullscreen_modes(
        &mut self,
    ) -> VidextResult<impl IntoIterator<Item = m64prs_sys::Size2D>> {
        VidextResult::<[m64prs_sys::Size2D; 0]>::Err(M64PError::Unsupported)
    }

    unsafe fn list_fullscreen_rates(
        &mut self,
        size: m64prs_sys::Size2D,
    ) -> VidextResult<impl IntoIterator<Item = c_int>> {
        VidextResult::<[c_int; 0]>::Err(M64PError::Unsupported)
    }

    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: m64prs_sys::VideoMode,
        flags: m64prs_sys::VideoFlags,
    ) -> VidextResult<()> {
        match &mut self.graphics {
            GraphicsState::OpenGl(opengl_state @ Some(_)) => {
                let return_value: VidextResult<()>;
                (return_value, *opengl_state) = match opengl_state.take().unwrap() {
                    OpenGlState::Config(config_state) => 'config_state: {
                        // Get window request parameters from the config state
                        let (position, size, transparent) =
                            config_state.window_request_params(width, height);

                        // Request a subsurface
                        let subsurface_handle = match self
                            .request_mgr
                            .request(VidextRequest::CreateSubsurface {
                                position,
                                size: size.into(),
                                transparent,
                            })
                            .map_err(|_| M64PError::SystemFail)?
                        {
                            VidextResponse::Done => {
                                break 'config_state (
                                    Err(M64PError::SystemFail),
                                    Some(OpenGlState::Config(config_state)),
                                )
                            }
                            VidextResponse::NewSubsurface(subsurface_handle) => subsurface_handle,
                        };

                        // Initialize OpenGL with that subsurface
                        match config_state.setup_opengl_context(
                            subsurface_handle,
                            dpi::PhysicalSize::new(width, height).cast(),
                        ) {
                            Ok(active_state) => (Ok(()), Some(OpenGlState::Active(active_state))),
                            Err((error, config_state)) => {
                                (Err(error), Some(OpenGlState::Config(config_state)))
                            }
                        }
                    }
                    OpenGlState::Active(active) => (
                        Err(M64PError::InvalidState),
                        Some(OpenGlState::Active(active)),
                    ),
                };
                return_value
            }
            _ => Err(M64PError::Internal),
        }
    }

    unsafe fn set_video_mode_with_rate(
        &mut self,
        _width: c_int,
        _height: c_int,
        _refresh_rate: c_int,
        _bits_per_pixel: c_int,
        _screen_mode: m64prs_sys::VideoMode,
        _flags: m64prs_sys::VideoFlags,
    ) -> VidextResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn set_caption(&mut self, title: &CStr) -> VidextResult<()> {
        // no-op
        Ok(())
    }

    unsafe fn toggle_full_screen(&mut self) -> VidextResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn resize_window(&mut self, width: c_int, height: c_int) -> VidextResult<()> {
        Err(M64PError::Unsupported)
    }

    unsafe fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Active(active_state))) => {
                active_state.get_proc_address(symbol)
            },
            _ => null_mut()
        }
    }

    unsafe fn gl_set_attribute(
        &mut self,
        attr: m64prs_sys::GLAttribute,
        value: c_int,
    ) -> VidextResult<()> {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Config(config_state))) => {
                log::info!("Setting attribute {:?} = {:?}", attr, value);
                config_state.gl_set_attribute(attr, value)
            },
            _ => Err(M64PError::Internal)
        }
    }

    unsafe fn gl_get_attribute(&mut self, attr: m64prs_sys::GLAttribute) -> VidextResult<c_int> {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Active(active_state))) => {
                active_state.gl_get_attribute(attr)
            },
            _ => Err(M64PError::InvalidState)
        }
    }

    unsafe fn gl_swap_buffers(&mut self) -> VidextResult<()> {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Active(active_state))) => {
                active_state.swap_buffers()
            },
            _ => Err(M64PError::InvalidState)
        }
    }

    unsafe fn gl_get_default_framebuffer(&mut self) -> u32 {
        0
    }

    unsafe fn vk_get_surface(&mut self, _inst: &vk::Instance) -> VidextResult<vk::SurfaceKHR> {
        Err(M64PError::Unsupported)
    }

    unsafe fn vk_get_instance_extensions(&mut self) -> VidextResult<&'static [*const c_char]> {
        Err(M64PError::Unsupported)
    }
}
