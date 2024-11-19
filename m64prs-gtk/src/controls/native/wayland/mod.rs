use std::{ffi::c_void, ptr::NonNull, sync::Arc};

use gdk::prelude::*;
use glutin::display::DisplayApiPreference;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WaylandDisplayHandle,
    WaylandWindowHandle, WindowHandle,
};
use state::{DisplayState, WaylandDisplayExt, WaylandSurfaceExt};
use wayland_client::{
    protocol::{wl_subsurface::WlSubsurface, wl_surface::WlSurface},
    Proxy,
};

use super::{PlatformSubsurface, SubsurfaceAttributes};

mod macros;
mod state;

pub(super) struct WaylandSubsurface {
    display_state: Arc<DisplayState>,
    parent_surface: WlSurface,
    surface: WlSurface,
    subsurface: WlSubsurface,
    transparent: bool,
}

impl WaylandSubsurface {
    pub(super) fn new(
        gdk_surface: &gdk_wayland::WaylandSurface,
        attrs: SubsurfaceAttributes,
    ) -> Self {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_wayland::WaylandDisplay>()
            .unwrap();
        let st = gdk_display.display_state();

        let mut queue = st.queue.write().unwrap();
        let qh = queue.handle();

        let size: dpi::PhysicalSize<u32> = attrs.surface_size;
        let position: dpi::PhysicalPosition<i32> = attrs.position;

        let surface = st.compositor.create_surface(&qh, ());
        let parent_surface = gdk_surface.wl_surface();

        let subsurface = st
            .subcompositor
            .get_subsurface(&surface, &parent_surface, &qh, ());
        subsurface.set_desync();
        subsurface.set_position(position.x, position.y);

        let transparent = attrs.transparent;

        {
            let input_region = st.compositor.create_region(&qh, ());
            input_region.subtract(0, 0, size.width as i32, size.height as i32);
            surface.set_input_region(Some(&input_region));

            if !transparent {
                let opaque_region = st.compositor.create_region(&qh, ());
                opaque_region.add(0, 0, size.width as i32, size.height as i32);
                surface.set_opaque_region(Some(&opaque_region));
            }
        }

        subsurface.set_position(position.x, position.y);
        surface.commit();
        parent_surface.commit();

        queue.roundtrip();
        drop(queue);

        Self {
            display_state: st,
            parent_surface,
            surface,
            subsurface,
            transparent,
        }
    }
}

impl Drop for WaylandSubsurface {
    fn drop(&mut self) {
        log::info!("destroying wl_surface/wl_subsurface");
        self.subsurface.destroy();
        self.surface.destroy();
    }
}

impl PlatformSubsurface for WaylandSubsurface {
    fn set_position(&self, position: dpi::PhysicalPosition<i32>) {
        self.subsurface.set_position(position.x, position.y);
        self.parent_surface.commit();
    }

    fn set_size(&self, size: dpi::PhysicalSize<u32>) {
        if !self.transparent {
            let st = &self.display_state;
            let queue = st.queue.read().unwrap();
            let qh = queue.handle();

            let opaque_region = st.compositor.create_region(&qh, ());
            opaque_region.add(0, 0, size.width as i32, size.height as i32);
            self.surface.set_opaque_region(Some(&opaque_region));
        }
    }

    fn display_handle_src(&self) -> &dyn raw_window_handle::HasDisplayHandle {
        self
    }

    fn window_handle_src(&self) -> &dyn raw_window_handle::HasWindowHandle {
        self
    }

    fn gl_api_preference(&self) -> DisplayApiPreference {
        DisplayApiPreference::Egl
    }
}

impl HasDisplayHandle for WaylandSubsurface {
    fn display_handle<'a>(&'a self) -> Result<DisplayHandle<'a>, HandleError> {
        let ptr = NonNull::new(self.display_state.display.id().as_ptr() as *mut c_void)
            .ok_or(HandleError::Unavailable)?;

        let raw_handle = WaylandDisplayHandle::new(ptr);

        unsafe { Ok(DisplayHandle::borrow_raw(raw_handle.into())) }
    }
}

impl HasWindowHandle for WaylandSubsurface {
    fn window_handle<'a>(&'a self) -> Result<WindowHandle<'a>, HandleError> {
        let ptr = NonNull::new(self.surface.id().as_ptr() as *mut c_void)
            .ok_or(HandleError::Unavailable)?;

        let raw_handle = WaylandWindowHandle::new(ptr);

        unsafe { Ok(WindowHandle::borrow_raw(raw_handle.into())) }
    }
}
