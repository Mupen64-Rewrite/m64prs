use std::{
    ffi::{c_char, c_int, c_void, CStr, CString}, mem, thread::ThreadId
};

use ash::vk::{self, Handle};
use m64prs_sys::{
    Error as SysError, GLAttribute, RenderMode, Size2D, VideoExtensionFunctions, VideoFlags,
    VideoMode,
};
use std::sync::Mutex;

use crate::{error::M64PError, types::VideoExtension};

use super::{core_fn, Core};

impl Core {
    pub fn override_vidext<V: VideoExtension + 'static>(&mut self, vidext: V) -> Result<(), M64PError> {
        let mut vidext_box = VIDEXT_BOX.lock().unwrap();
        *vidext_box = Some(Box::new(VideoExtensionWrapper(vidext)));
        drop(vidext_box);

        core_fn(unsafe { self.api.base.override_vidext(&VIDEXT_TABLE as *const _ as *mut _) })
    }
}

trait VideoExtensionDyn: Send {
    /// Initializes the video extension with the specified graphics API.
    unsafe fn init_with_render_mode(&mut self, mode: RenderMode) -> m64prs_sys::Error;
    /// Shuts down the video extension.
    unsafe fn quit(&mut self) -> m64prs_sys::Error;

    /// Lists the available resolutions when rendering in full screen.
    unsafe fn list_fullscreen_modes(
        &mut self,
        modes: *mut Size2D,
        count: *mut c_int,
    ) -> m64prs_sys::Error;
    /// Lists the available refresh rates for a specific fullscreen resolution.
    unsafe fn list_fullscreen_rates(
        &mut self,
        size: Size2D,
        rates: *mut c_int,
        count: *mut c_int,
    ) -> m64prs_sys::Error;

    /// Sets up a render context with the specified dimensions and current OpenGL attributes.
    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: c_int,
        flags: c_int,
    ) -> m64prs_sys::Error;
    /// Sets up a render context with the specified dimensions, refresh rate, and current OpenGL attributes.
    unsafe fn set_video_mode_with_rate(
        &mut self,
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: c_int,
        flags: c_int,
    ) -> m64prs_sys::Error;

    /// Sets the window title.
    unsafe fn set_caption(&mut self, title: *const c_char) -> m64prs_sys::Error;
    /// Toggles fullscreen.
    unsafe fn toggle_full_screen(&mut self) -> m64prs_sys::Error;
    /// Resizes the render context to the specified width and height.
    unsafe fn resize_window(&mut self, width: c_int, height: c_int) -> m64prs_sys::Error;

    /// Grabs an OpenGL function with the specified name.
    unsafe fn gl_get_proc_address(&mut self, symbol: *const c_char) -> Option<unsafe extern "C" fn()>;
    /// Sets an OpenGL attribute. This is called before [`VideoExtension::set_video_mode`].
    unsafe fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> m64prs_sys::Error;
    /// Gets an OpenGL attribute. This is generally called after [`VideoExtension::set_video_mode`].
    unsafe fn gl_get_attribute(
        &mut self,
        attr: GLAttribute,
        value: *mut c_int,
    ) -> m64prs_sys::Error;
    /// Swaps buffers on the current render context.
    unsafe fn gl_swap_buffers(&mut self) -> m64prs_sys::Error;
    /// Gets the default FBO for this render context.
    unsafe fn gl_get_default_framebuffer(&mut self) -> u32;

    /// Acquires a Vulkan surface from the window.
    unsafe fn vk_get_surface(
        &mut self,
        surface: *mut *mut c_void,
        instance: *mut c_void,
    ) -> m64prs_sys::Error;
    /// Lists the extensions needed to use [`VideoExtension::vk_get_surface`]
    unsafe fn vk_get_instance_extensions(
        &mut self,
        extensions: *mut *mut *const c_char,
        count: *mut u32,
    ) -> m64prs_sys::Error;
}

struct VideoExtensionWrapper<V: VideoExtension>(V);

unsafe impl<V: VideoExtension> Send for VideoExtensionWrapper<V> {}

impl<V: VideoExtension> VideoExtensionDyn for VideoExtensionWrapper<V> {
    #[inline(always)]
    unsafe fn init_with_render_mode(&mut self, mode: RenderMode) -> SysError {
        match self.0.init_with_render_mode(mode) {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn quit(&mut self) -> SysError {
        match self.0.quit() {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn list_fullscreen_modes(&mut self, modes: *mut Size2D, count: *mut c_int) -> SysError {
        match self.0.list_fullscreen_modes() {
            Ok(mode_iter) => unsafe {
                let slice = std::slice::from_raw_parts_mut(modes, *count as usize);
                let mut copy_count: usize = 0;
                for (dst, src) in std::iter::zip(slice.iter_mut(), mode_iter) {
                    *dst = src;
                    copy_count += 1;
                }

                *count = copy_count as c_int;

                SysError::Success
            },
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn list_fullscreen_rates(
        &mut self,
        size: Size2D,
        rates: *mut c_int,
        count: *mut c_int,
    ) -> SysError {
        match self.0.list_fullscreen_rates(size) {
            Ok(rate_iter) => {
                let slice = std::slice::from_raw_parts_mut(rates, *count as usize);
                let mut copy_count: usize = 0;
                for (dst, src) in std::iter::zip(slice.iter_mut(), rate_iter) {
                    *dst = src;
                    copy_count += 1;
                }

                *count = copy_count as c_int;
                SysError::Success
            }
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: c_int,
        flags: c_int,
    ) -> SysError {
        let screen_mode = match VideoMode::try_from(screen_mode as u32) {
            Ok(value) => value,
            Err(_) => return SysError::InputAssert
        };
        let flags = VideoFlags::from_bits_retain(flags as u32);

        match self
            .0
            .set_video_mode(width, height, bits_per_pixel, screen_mode, flags)
        {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn set_video_mode_with_rate(
        &mut self,
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: c_int,
        flags: c_int,
    ) -> SysError {
        let screen_mode = match VideoMode::try_from(screen_mode as u32) {
            Ok(value) => value,
            Err(_) => return SysError::InputAssert
        };
        let flags = VideoFlags::from_bits_retain(flags as u32);

        match self.0.set_video_mode_with_rate(
            width,
            height,
            refresh_rate,
            bits_per_pixel,
            screen_mode,
            flags,
        ) {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn set_caption(&mut self, title: *const c_char) -> SysError {
        match self.0.set_caption(CStr::from_ptr(title)) {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn toggle_full_screen(&mut self) -> SysError {
        match self.0.toggle_full_screen() {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn resize_window(&mut self, width: c_int, height: c_int) -> SysError {
        match self.0.resize_window(width, height) {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn gl_get_proc_address(&mut self, symbol: *const c_char) -> Option<unsafe extern "C" fn()> {
        mem::transmute(self.0.gl_get_proc_address(CStr::from_ptr(symbol)))
    }

    #[inline(always)]
    unsafe fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> SysError {
        match self.0.gl_set_attribute(attr, value) {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn gl_get_attribute(&mut self, attr: GLAttribute, value: *mut c_int) -> SysError {
        match self.0.gl_get_attribute(attr) {
            Ok(result) => unsafe {
                *value = result;
                SysError::Success
            },
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn gl_swap_buffers(&mut self) -> SysError {
        match self.0.gl_swap_buffers() {
            Ok(()) => SysError::Success,
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn gl_get_default_framebuffer(&mut self) -> u32 {
        self.0.gl_get_default_framebuffer()
    }

    #[inline(always)]
    unsafe fn vk_get_surface(
        &mut self,
        surface: *mut *mut c_void,
        instance: *mut c_void,
    ) -> SysError {
        match self.0.vk_get_surface(vk::Instance::from_raw(
            (instance as usize).try_into().unwrap(),
        )) {
            Ok(surface_obj) => {
                *surface = usize::try_from(surface_obj.as_raw()).unwrap() as *mut c_void;
                SysError::Success
            }
            Err(error) => error.into(),
        }
    }

    #[inline(always)]
    unsafe fn vk_get_instance_extensions(
        &mut self,
        extensions: *mut *mut *const c_char,
        count: *mut u32,
    ) -> SysError {
        match self.0.vk_get_instance_extensions() {
            Ok(extension_span) => {
                *count = extension_span.len().try_into().unwrap();
                *extensions = extension_span.as_ptr() as *mut *const c_char;
                SysError::Success
            }
            Err(error) => error.into(),
        }
    }
}

static VIDEXT_BOX: Mutex<Option<Box<dyn VideoExtensionDyn>>> = Mutex::new(None);

macro_rules! extern_c_fn {
    ( | $($param:ident : $ptype:ty),* $(,)? | $(-> $rtype:ty)? { $($code:tt)* } ) => {
        {
            {
                unsafe extern "C" fn f($($param: $ptype),*) $(-> $rtype)? {
                    $($code)*
                }
                f
            }
        }

    };
    ( || $(-> $rtype:ty)? { $($code:tt)* } ) => {
        {
            {
                unsafe extern "C" fn f() $(-> $rtype)? {
                    $($code)*
                }
                f
            }
        }
    };
}

static VIDEXT_TABLE: VideoExtensionFunctions = VideoExtensionFunctions {
    Functions: 17,
    VidExtFuncInit: Some(extern_c_fn!(|| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .init_with_render_mode(RenderMode::OpenGl)
    })),
    VidExtFuncQuit: Some(extern_c_fn!(|| -> SysError {
        VIDEXT_BOX.lock().unwrap().as_mut().unwrap().quit()
    })),
    VidExtFuncListModes: Some(extern_c_fn!(|modes: *mut Size2D,
                                          count: *mut c_int|
     -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .list_fullscreen_modes(modes, count)
    })),
    VidExtFuncListRates: Some(extern_c_fn!(|size: Size2D,
                                          rates: *mut c_int,
                                          count: *mut c_int|
     -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .list_fullscreen_rates(size, rates, count)
    })),
    VidExtFuncSetMode: Some(extern_c_fn!(|width: c_int,
                                        height: c_int,
                                        bits_per_pixel: c_int,
                                        screen_mode: c_int,
                                        flags: c_int|
     -> SysError {
        

        VIDEXT_BOX.lock().unwrap().as_mut().unwrap().set_video_mode(
            width,
            height,
            bits_per_pixel,
            screen_mode,
            flags,
        )
    })),
    VidExtFuncSetModeWithRate: Some(extern_c_fn!(|width: c_int,
                                                height: c_int,
                                                refresh_rate: c_int,
                                                bits_per_pixel: c_int,
                                                screen_mode: c_int,
                                                flags: c_int|
     -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_video_mode_with_rate(
                width,
                height,
                refresh_rate,
                bits_per_pixel,
                screen_mode,
                flags,
            )
    })),
    VidExtFuncGLGetProc: Some(extern_c_fn!(|symbol: *const c_char| -> Option<unsafe extern "C" fn()> {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .gl_get_proc_address(symbol)
    })),
    VidExtFuncGLSetAttr: Some(extern_c_fn!(|attr: GLAttribute, value: c_int| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .gl_set_attribute(attr, value)
    })),
    VidExtFuncGLGetAttr: Some(extern_c_fn!(|attr: GLAttribute,
                                          value: *mut c_int|
     -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .gl_get_attribute(attr, value)
    })),
    VidExtFuncGLSwapBuf: Some(extern_c_fn!(|| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .gl_swap_buffers()
    })),
    VidExtFuncSetCaption: Some(extern_c_fn!(|title: *const c_char| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_caption(title)
    })),
    VidExtFuncToggleFS: Some(extern_c_fn!(|| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .toggle_full_screen()
    })),
    VidExtFuncResizeWindow: Some(extern_c_fn!(|width: c_int, height: c_int| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .resize_window(width, height)
    })),
    VidExtFuncGLGetDefaultFramebuffer: Some(extern_c_fn!(|| -> u32 {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .gl_get_default_framebuffer()
    })),
    VidExtFuncInitWithRenderMode: Some(extern_c_fn!(|render_mode: RenderMode| -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .init_with_render_mode(render_mode)
    })),
    VidExtFuncVKGetSurface: Some(extern_c_fn!(|surface: *mut *mut c_void,
                                             instance: *mut c_void|
     -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .vk_get_surface(surface, instance)
    })),
    VidExtFuncVKGetInstanceExtensions: Some(extern_c_fn!(|extensions: *mut *mut *const c_char,
                                                        count: *mut u32|
     -> SysError {
        VIDEXT_BOX
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .vk_get_instance_extensions(extensions, count)
    })),
};