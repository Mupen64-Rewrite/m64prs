
use glib::object::{Cast, ObjectExt};
use glutin::display::DisplayApiPreference;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[cfg(all(target_os = "linux", feature = "wayland"))]
mod wayland;
#[cfg(all(target_os = "windows"))]
mod windows;
#[cfg(all(target_os = "linux", feature = "x11"))]
mod xcb;

slotmap::new_key_type! {
    pub struct NativeViewKey;
}

#[derive(Debug, Clone)]
pub enum StackOrder {
    StackAbove(NativeViewKey),
    StackBelow(NativeViewKey),
}

#[derive(Debug, Clone)]
pub struct NativeViewAttributes {
    pub surface_size: dpi::PhysicalSize<u32>,
    pub position: dpi::PhysicalPosition<i32>,
    pub stack_order: Option<StackOrder>,
    pub transparent: bool,
}

impl Default for NativeViewAttributes {
    fn default() -> Self {
        Self {
            surface_size: dpi::PhysicalSize::new(100, 100),
            position: dpi::PhysicalPosition::new(0, 0),
            stack_order: None,
            transparent: false,
        }
    }
}

impl NativeViewAttributes {
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
    pub fn with_stack_order(mut self, order: StackOrder) -> Self {
        self.stack_order = Some(order);
        self
    }
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
}

pub trait NativeCompositor {
    /// Creates a new view.
    fn new_view(&mut self, attrs: NativeViewAttributes) -> Box<dyn NativeView>;
    /// Closes a currently open view.
    fn delete_view(&mut self, view: NativeViewKey);

    /// Sets the bounds of a view within the compositor.
    fn set_view_bounds(
        &mut self,
        key: NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    );
    /// Restacks a view relative to another.
    fn restack_view(&mut self, key: NativeViewKey, stack_order: StackOrder);

    /// Computes the total bounds of all views in the compositor.
    fn total_bounds(&self) -> dpi::PhysicalSize<u32>;

    /// Sets the position of the compositor with respect tothe parent surface.
    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>);
    /// Maps or unmaps the compositor.
    fn set_mapped(&mut self, mapped: bool);

    /// Returns the conversion factor between compositor coordinates and surface coordinates.
    fn scale_factor(&self) -> f64;
}

pub trait NativeView: Send + Sync {
    /// Returns a unique identifier for this view.
    fn key(&self) -> NativeViewKey;
    /// Returns an object owning the display handle.
    fn display_handle_src(&self) -> &dyn HasDisplayHandle;
    /// Returns an object owning the window handle.
    fn window_handle_src(&self) -> &dyn HasWindowHandle;
    /// Returns the preferred OpenGL API for this surface.
    fn gl_api_preference(&self) -> DisplayApiPreference;
}

impl dyn NativeCompositor {
    /// Creates a new compositor. The compositor is initially unmapped and must be mapped
    /// by a call to [`NativeCompositor::set_mapped`].
    pub fn new(
        surface: gdk::Surface,
        position: dpi::PhysicalPosition<i32>,
    ) -> Box<dyn NativeCompositor> {
        #[cfg(target_os = "windows")]
        {
            if surface.is::<gdk_win32::Win32Surface>() {
                todo!()
            }
            unreachable!()
        }
        #[cfg(target_os = "linux")]
        {
            #[cfg(feature = "x11")]
            if surface.is::<gdk_x11::X11Surface>() {
                todo!()
            }
            #[cfg(feature = "wayland")]
            if surface.is::<gdk_wayland::WaylandSurface>() {
                return Box::new(wayland::WaylandCompositor::new(
                    surface.downcast_ref().unwrap(),
                    position,
                ))
            }
            unreachable!()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        compile_error!("Platform is currently not supported");
    }
}

impl HasWindowHandle for dyn NativeView {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        return self.window_handle_src().window_handle()
    }
}
impl HasDisplayHandle for dyn NativeView {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        return self.display_handle_src().display_handle()
    }
}