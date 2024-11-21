use glutin::{
    api::egl::{
        config::Config as EGLConfig, context::PossiblyCurrentContext as EGLPossiblyCurrentContext,
        display::Display as EGLDisplay,
    },
    config::GetGlConfig,
    context::AsRawContext,
    display::{AsRawDisplay, GetGlDisplay, RawDisplay},
};
use glutin_egl_sys::egl::{self, types::EGLClientBuffer};
use std::{ffi::c_void, ptr::null};
use wayland_backend::client::ObjectId;

use crate::utils::gl::types::GLenum;

use wayland_client::{protocol::wl_buffer::WlBuffer, Connection, Proxy};

pub struct EGLImage {
    display: EGLDisplay,
    config: EGLConfig,
    raw_image: *const c_void,
}

mod sealed {
    pub trait Sealed {}
}

pub trait EGLContextExt: sealed::Sealed {
    unsafe fn create_image_renderbuffer(
        &self,
        rbo: GLenum,
    ) -> Result<EGLImage, glutin::error::Error>;
}

impl sealed::Sealed for EGLPossiblyCurrentContext {}
impl EGLContextExt for EGLPossiblyCurrentContext {
    unsafe fn create_image_renderbuffer(
        &self,
        rbo: GLenum,
    ) -> Result<EGLImage, glutin::error::Error> {
        let display = self.display();

        let egl = display.egl();
        let display_ptr = match display.raw_display() {
            RawDisplay::Egl(ptr) => ptr,
            _ => unreachable!(),
        };
        let context_ptr = match self.raw_context() {
            glutin::context::RawContext::Egl(ptr) => ptr,
            _ => unreachable!(),
        };

        let image = unsafe {
            egl.CreateImage(
                display_ptr,
                context_ptr,
                egl::GL_RENDERBUFFER,
                rbo as usize as EGLClientBuffer,
                null(),
            )
        };
        if image.is_null() {
            check_error(&display)?;
        }

        Ok(EGLImage {
            display,
            config: self.config(),
            raw_image: image,
        })
    }
}

impl EGLImage {
    pub fn config(&self) -> EGLConfig {
        self.config.clone()
    }

    pub fn display(&self) -> EGLDisplay {
        self.display.clone()
    }

    pub unsafe fn get_wayland_buffer(&self, conn: &Connection) -> WlBuffer {
        let display = self.display();

        let egl = display.egl();
        let display_ptr = match display.raw_display() {
            RawDisplay::Egl(ptr) => ptr,
            _ => unreachable!(),
        };

        unsafe {
            let wl_buffer_ptr = egl.CreateWaylandBufferFromImageWL(display_ptr, self.raw_image);
            let wl_buffer_id = ObjectId::from_ptr(WlBuffer::interface(), wl_buffer_ptr as *mut _)
                .expect("eglCreateWaylandBufferFromImageWL should return a wl_buffer*");

            WlBuffer::from_id(conn, wl_buffer_id).expect("eglCreateWaylandBuffer")
        }
    }
}

impl Drop for EGLImage {
    fn drop(&mut self) {
        let display = self.display();

        let egl = display.egl();
        let display_ptr = match display.raw_display() {
            RawDisplay::Egl(ptr) => ptr,
            _ => unreachable!(),
        };

        unsafe {
            egl.DestroyImage(display_ptr, self.raw_image);
        }
    }
}

fn check_error(disp: &EGLDisplay) -> Result<(), glutin::error::Error> {
    use glutin::error::ErrorKind;
    let egl = disp.egl();
    unsafe {
        let raw_code = egl.GetError() as egl::types::EGLenum;
        let kind = match raw_code {
            egl::SUCCESS => return Ok(()),
            egl::NOT_INITIALIZED => ErrorKind::InitializationFailed,
            egl::BAD_ACCESS => ErrorKind::BadAccess,
            egl::BAD_ALLOC => ErrorKind::OutOfMemory,
            egl::BAD_ATTRIBUTE => ErrorKind::BadAttribute,
            egl::BAD_CONTEXT => ErrorKind::BadContext,
            egl::BAD_CONFIG => ErrorKind::BadConfig,
            egl::BAD_CURRENT_SURFACE => ErrorKind::BadCurrentSurface,
            egl::BAD_DISPLAY => ErrorKind::BadDisplay,
            egl::BAD_SURFACE => ErrorKind::BadSurface,
            egl::BAD_MATCH => ErrorKind::BadMatch,
            egl::BAD_PARAMETER => ErrorKind::BadParameter,
            egl::BAD_NATIVE_PIXMAP => ErrorKind::BadNativePixmap,
            egl::BAD_NATIVE_WINDOW => ErrorKind::BadNativeWindow,
            egl::CONTEXT_LOST => ErrorKind::ContextLost,
            _ => ErrorKind::Misc,
        };

        Err(kind.into())
    }
}
