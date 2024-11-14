use std::{ffi::{c_char, c_int, c_void, CStr}, fmt::Debug, sync::{atomic::{self, AtomicUsize}, mpsc}, time::Duration};

use ash::vk;
use graphene::{Point, Size};
use m64prs_core::{error::M64PError, vidext::{VideoExtension, VidextResult}};
use relm4::ComponentSender;

use crate::controls::SubsurfaceHandle;

#[derive(Debug)]
pub enum VidextRequest {
    EnterGameView,
    ExitGameView,
    CreateSubsurface {
        position: Point,
        size: Size,
        transparent: bool,
    },
    FreeSubsurface(SubsurfaceHandle)
}

#[derive(Debug)]
pub enum VidextResponse {
    Done,
    NewSubsurface(SubsurfaceHandle),
}

pub struct VideoExtensionState {
    uid_counter: AtomicUsize,
    outbound: ComponentSender<super::Model>,
    inbound: mpsc::Receiver<(usize, VidextResponse)>
}

impl Debug for VideoExtensionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoExtensionState").finish()
    }
}

impl VideoExtensionState {
    /// Constructs a 
    pub(super) fn new(sender: ComponentSender<super::Model>) -> (Self, mpsc::Sender<(usize, VidextResponse)>) {
        let (tx, rx) = mpsc::channel();

        let inst = Self {
            uid_counter: AtomicUsize::new(0),
            outbound: sender,
            inbound: rx,
        };
        (inst, tx)
    }

    fn request(&mut self, req: VidextRequest) -> Result<VidextResponse, mpsc::RecvError> {
        // get request ID (used to verify that the request is indeed the correct one)
        let id = self.uid_counter.fetch_add(1, atomic::Ordering::AcqRel);
        // send out the request
        self.outbound.output(super::Response::VidextRequest(id, req))
            .expect("Sender should still be valid");
        // wait for a reply
        self.inbound.recv().map(|(reply_id, resp)| {
            assert!(reply_id == id);
            resp
        })
    }
}

// TODO: implement vidext
#[allow(unused)]
impl VideoExtension for VideoExtensionState {
    unsafe fn init_with_render_mode(&mut self, mode: m64prs_sys::RenderMode) -> VidextResult<()> {
        self.request(VidextRequest::EnterGameView).map_err(|_| M64PError::Internal)?;

        Ok(())
    }

    unsafe fn quit(&mut self) -> VidextResult<()> {
        todo!()
    }

    unsafe fn list_fullscreen_modes(&mut self) -> VidextResult<impl IntoIterator<Item = m64prs_sys::Size2D>> {
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
        todo!()
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
        todo!()
    }

    unsafe fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void {
        todo!()
    }

    unsafe fn gl_set_attribute(&mut self, attr: m64prs_sys::GLAttribute, value: c_int) -> VidextResult<()> {
        todo!()
    }

    unsafe fn gl_get_attribute(&mut self, attr: m64prs_sys::GLAttribute) -> VidextResult<c_int> {
        todo!()
    }

    unsafe fn gl_swap_buffers(&mut self) -> VidextResult<()> {
        todo!()
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