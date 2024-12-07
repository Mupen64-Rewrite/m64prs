use std::{
    any::Any,
    ffi::{c_char, c_int, c_void, CStr},
    fmt::Debug,
    ptr::null_mut,
};

use ash::vk;
use glib::SendWeakRef;
use m64prs_core::{
    error::M64PError,
    vidext::{VideoExtension, VidextResult},
};
use m64prs_sys::RenderMode;
use opengl::OpenGlState;
use pollster::FutureExt;

use crate::ui::main_window::{enums::MainViewState, MainWindow};

mod opengl;

enum GraphicsState {
    Uninit,
    OpenGl(Option<OpenGlState>),
    // Vulkan
}

pub struct VideoExtensionParameters {
    main_window_ref: SendWeakRef<MainWindow>,
}

impl VideoExtensionParameters {
    pub fn new(main_window_ref: SendWeakRef<MainWindow>) -> Self {
        Self { main_window_ref }
    }
}

pub struct VideoExtensionState {
    main_window_ref: SendWeakRef<MainWindow>,
    graphics: GraphicsState,
}

impl Debug for VideoExtensionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoExtensionState").finish()
    }
}

impl VideoExtensionState {
    fn new(parameters: &VideoExtensionParameters) -> Self {
        Self {
            main_window_ref: parameters.main_window_ref.clone(),
            graphics: GraphicsState::Uninit,
        }
    }
}

// TODO: implement vidext
#[allow(unused)]
impl VideoExtension for VideoExtensionState {
    unsafe fn init_with_render_mode(mode: RenderMode, context: &mut dyn Any) -> VidextResult<Self> {
        let context = context
            .downcast_mut::<VideoExtensionParameters>()
            .expect("expected Option<VideoExtensionParameters> from downcast");

        match mode {
            RenderMode::OpenGl => {
                let mut inst = Self::new(&context);
                // Request the GUI to switch to the game view.
                log::info!("Requesting switch to game view");
                {
                    let main_window_ref = inst.main_window_ref.clone();
                    glib::spawn_future(async move {
                        let main_window = main_window_ref
                            .upgrade()
                            .expect("Main window should be active");

                        main_window.set_current_view(MainViewState::GameView);
                    })
                    .block_on();
                }

                log::info!("Init successful");
                inst.graphics = GraphicsState::OpenGl(Some(OpenGlState::default()));
                Ok(inst)
            }
            RenderMode::Vulkan => Err(M64PError::Unsupported),
        }
    }

    unsafe fn quit(self, context: &mut dyn Any) -> VidextResult<()> {
        let view_key = match &self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Active(active_state))) => {
                Some(active_state.native_view_key())
            }
            _ => None,
        };

        {
            let main_window_ref = self.main_window_ref.clone();
            glib::spawn_future(async move {
                let main_window = main_window_ref
                    .upgrade()
                    .expect("Main window should be active");

                if let Some(view_key) = view_key {
                    main_window.comp_del_view(view_key);
                }

                main_window.set_current_view(MainViewState::RomBrowser);
            })
            .block_on();
        }

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
        if width <= 0 || height <= 0 {
            log::error!("set_video_mode: width and height must be non-negative");
            return Err(M64PError::InputAssert);
        }
        match &mut self.graphics {
            GraphicsState::OpenGl(opengl_state @ Some(_)) => {
                let return_value: VidextResult<()>;
                (return_value, *opengl_state) = match opengl_state.take().unwrap() {
                    OpenGlState::Config(config_state) => 'config_state: {
                        // Get window request parameters from the config state
                        let attrs = config_state.window_request_params(width, height);

                        // Request a subsurface
                        let subsurface_handle = {
                            let main_window_ref = self.main_window_ref.clone();
                            glib::spawn_future(async move {
                                let main_window = main_window_ref
                                    .upgrade()
                                    .expect("Main window should be active");

                                main_window.comp_new_view(attrs)
                            })
                            .block_on()
                            .map_err(|_| M64PError::Internal)?
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
            }
            _ => null_mut(),
        }
    }

    unsafe fn gl_set_attribute(
        &mut self,
        attr: m64prs_sys::GLAttribute,
        value: c_int,
    ) -> VidextResult<()> {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Config(config_state))) => {
                config_state.gl_set_attribute(attr, value)
            }
            _ => Err(M64PError::Internal),
        }
    }

    unsafe fn gl_get_attribute(&mut self, attr: m64prs_sys::GLAttribute) -> VidextResult<c_int> {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Active(active_state))) => {
                active_state.gl_get_attribute(attr)
            }
            _ => Err(M64PError::InvalidState),
        }
    }

    unsafe fn gl_swap_buffers(&mut self) -> VidextResult<()> {
        match &mut self.graphics {
            GraphicsState::OpenGl(Some(OpenGlState::Active(active_state))) => {
                active_state.swap_buffers()
            }
            _ => Err(M64PError::InvalidState),
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
