use std::ffi::{c_int, c_void, CStr};

use dpi::{PhysicalPosition, PhysicalSize, Position};
use glib::object::{Cast, ObjectExt};
use glutin::{
    config::{Config as GlutinConfig, ConfigTemplate},
    context::{ContextAttributes, PossiblyCurrentContext},
    surface::{GlSurface, Surface as GlutinSurface, SurfaceAttributes, WindowSurface},
};
use gtk::gdk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[cfg(all(target_os = "linux", feature = "wayland"))]
mod wayland;
#[cfg(all(target_os = "windows"))]
mod windows;
#[cfg(all(target_os = "linux", feature = "x11"))]
mod x11;

pub mod conv;

/// Trait implementing a subsurface attached to a [`gdk::Surface`].
pub trait PlatformSubsurface: Send + Sync {
    /// Sets the position of the subsurface relative to the parent.
    fn set_position(&self, position: PhysicalPosition<i32>);
    /// Sets the size of the subsurface.
    fn set_size(&self, size: PhysicalSize<u32>);
    /// Returns an object owning the display handle.
    fn display_handle_src(&self) -> &dyn raw_window_handle::HasDisplayHandle;
    /// Returns an object owning the window handle.
    fn window_handle_src(&self) -> &dyn raw_window_handle::HasWindowHandle;
}

pub struct SubsurfaceAttributes {
    pub surface_size: dpi::PhysicalSize<u32>,
    pub position: dpi::PhysicalPosition<i32>,
    pub transparent: bool,
}

impl Default for SubsurfaceAttributes {
    fn default() -> Self {
        Self {
            surface_size: dpi::PhysicalSize::new(100, 100),
            position: dpi::PhysicalPosition::new(0, 0),
            transparent: false,
        }
    }
}

impl SubsurfaceAttributes {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_surface_size<S: Into<dpi::PhysicalSize<u32>>>(mut self, size: S) -> Self {
        self.surface_size = size.into();
        self
    }
    pub fn with_position<P: Into<dpi::PhysicalPosition<i32>>>(mut self, position: P) -> Self {
        self.position = position.into();
        self
    }
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
}

impl dyn PlatformSubsurface {
    pub fn new(window: &gdk::Surface, attributes: SubsurfaceAttributes) -> Box<Self> {
        #[cfg(target_os = "windows")]
        {}
        #[cfg(target_os = "linux")]
        {
            #[cfg(feature = "wayland")]
            if window.is::<gdk_wayland::WaylandSurface>() {
                return Box::new(wayland::WaylandSubsurface::new(
                    window.downcast_ref().unwrap(),
                    attributes,
                ));
            }
            #[cfg(feature = "x11")]
            if window.is::<gdk_x11::X11Surface>() {
                return Box::new(x11::X11Subsurface::new(
                    window.downcast_ref().unwrap(),
                    attributes,
                ));
            }
        }

        panic!(
            "not implemented: platform subsurface for {}",
            window.type_().name()
        );
    }
}

impl HasDisplayHandle for dyn PlatformSubsurface {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.display_handle_src().display_handle()
    }
}

impl HasWindowHandle for dyn PlatformSubsurface {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.window_handle_src().window_handle()
    }
}
