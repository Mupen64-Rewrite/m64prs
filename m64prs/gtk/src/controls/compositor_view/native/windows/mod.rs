use std::{ffi::c_void, num::NonZero, ptr::null_mut, sync::Arc};

use gdk::prelude::*;
use glutin::display::DisplayApiPreference;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, Win32WindowHandle, WindowHandle,
};
use slotmap::DenseSlotMap;
use state::{DisplayState, Win32DisplayExt};
use windows::{
    core::{w, IUnknown, Interface},
    Win32::{
        Foundation::{BOOL, COLORREF, HMODULE, HWND, POINT},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_UNKNOWN,
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_SDK_VERSION,
            },
            DirectComposition::{
                DCompositionCreateDevice, IDCompositionDevice, IDCompositionScaleTransform,
                IDCompositionSurface, IDCompositionTarget, IDCompositionVisual,
            },
            Dwm::{
                DwmEnableBlurBehindWindow, DwmSetWindowAttribute, DWMWA_CLOAK, DWM_BB_BLURREGION,
                DWM_BB_ENABLE, DWM_BLURBEHIND,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM},
                CreateDXGIFactory2, IDXGIAdapter, IDXGIDevice, IDXGIFactory2, IDXGISurface1,
                DXGI_CREATE_FACTORY_FLAGS,
            },
            Gdi::{self, CreateRectRgn},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, SetWindowPos, ShowWindow, HMENU, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOREPOSITION, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE,
            WINDOW_EX_STYLE, WS_CHILD, WS_DISABLED, WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
};

use super::{NativeCompositor, NativeView, NativeViewKey};

mod state;
mod util;

pub(super) struct WindowsCompositor {
    display_state: Arc<DisplayState>,
    views: DenseSlotMap<NativeViewKey, WindowsView>,

    current_bounds: dpi::PhysicalSize<u32>,
    mapped: bool,

    window: HWND,

    dx_factory: IDXGIFactory2,
    dx_adapter: IDXGIAdapter,

    d3d_device: ID3D11Device,
    dx_device: IDXGIDevice,

    dcomp_device: IDCompositionDevice,
    dcomp_target: IDCompositionTarget,

    root_visual: IDCompositionVisual,
    bg_visual: IDCompositionVisual,
    bg_surface: IDCompositionSurface,
    bg_transform: IDCompositionScaleTransform,
}

struct WindowsView {
    window: HWND,

    dcomp_device: IDCompositionDevice,
    visual: IDCompositionVisual,
    surface: IUnknown,

    position: dpi::PhysicalPosition<i32>,
    size: dpi::PhysicalSize<u32>,
}

struct WindowsViewHandle {
    view_key: NativeViewKey,
    display_state: Arc<DisplayState>,

    window: HWND,
}

impl WindowsCompositor {
    pub(super) fn new(
        gdk_surface: &gdk_win32::Win32Surface,
        position: dpi::PhysicalPosition<i32>,
    ) -> Self {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_win32::Win32Display>()
            .unwrap();
        let st = gdk_display.display_state();

        let parent_hwnd: HWND = HWND(gdk_surface.handle().0 as usize as *mut c_void);

        unsafe {
            Self::new_inner(st, parent_hwnd, position).expect("State creation should succeed")
        }
    }

    unsafe fn new_inner(
        st: Arc<DisplayState>,
        parent_hwnd: HWND,
        position: dpi::PhysicalPosition<i32>,
    ) -> Result<Self, windows::core::Error> {
        // Create the child window as the DComp root
        let window = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            state::COMP_WINDOW_CLASS,
            w!("output window"),
            WS_CHILD | WS_DISABLED,
            position.x,
            position.y,
            1,
            1,
            parent_hwnd,
            HMENU(null_mut()),
            st.hinstance,
            None,
        )?;

        let dx_factory: IDXGIFactory2 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
        let dx_adapter = dx_factory.EnumAdapters(0)?;

        let d3d_device = {
            let mut d3d_device: Option<ID3D11Device> = None;
            D3D11CreateDevice(
                &dx_adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE(null_mut()),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device as *mut _),
                None,
                None,
            )?;

            d3d_device.expect("d3d_device should be set after D3D11CreateDevice")
        };

        let dx_device = d3d_device.cast::<IDXGIDevice>()?;

        let dcomp_device: IDCompositionDevice = DCompositionCreateDevice(&dx_device)?;
        let dcomp_target = dcomp_device.CreateTargetForHwnd(window, true)?;

        // visual tree:
        // - root visual
        //   - background
        //   - window1
        //   - window2
        //   - etc.

        let bg_surface = Self::new_single_pixel_surface(&dcomp_device, util::rgb(0, 0, 0))?;
        let bg_transform = dcomp_device.CreateScaleTransform()?;
        bg_transform.SetCenterX2(0.0)?;
        bg_transform.SetCenterY2(0.0)?;
        bg_transform.SetScaleX2(1.0)?;
        bg_transform.SetScaleY2(1.0)?;

        let bg_visual = dcomp_device.CreateVisual()?;
        bg_visual.SetContent(&bg_surface)?;
        bg_visual.SetTransform(&bg_transform)?;

        let root_visual = dcomp_device.CreateVisual()?;
        root_visual.AddVisual(&bg_visual, true, None)?;

        dcomp_target.SetRoot(&root_visual)?;
        dcomp_device.Commit()?;

        Ok(Self {
            display_state: st,
            views: DenseSlotMap::with_key(),
            current_bounds: dpi::PhysicalSize::new(0, 0),
            mapped: false,
            window,
            dx_factory,
            dx_adapter,
            d3d_device,
            dx_device,
            dcomp_device,
            dcomp_target,
            root_visual,
            bg_visual,
            bg_surface,
            bg_transform,
        })
    }

    /// Creates a 1x1 DComp surface with a specific colour.
    unsafe fn new_single_pixel_surface(
        dcomp_device: &IDCompositionDevice,
        color: COLORREF,
    ) -> Result<IDCompositionSurface, windows::core::Error> {
        // Create a 1x1 surface
        let surface =
            dcomp_device.CreateSurface(1, 1, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_ALPHA_MODE_IGNORE)?;

        // Start drawing into it
        let mut point_offset = POINT::default();
        let dxgi_surface: IDXGISurface1 = surface.BeginDraw(None, &mut point_offset)?;

        // use GDI to render exactly one pixel
        {
            let surface_dc = dxgi_surface.GetDC(false)?;
            Gdi::SetPixel(surface_dc, 0, 0, color);
            dxgi_surface.ReleaseDC(None)?;
        }

        // Finish drawing into it
        let _ = surface.EndDraw();
        Ok(surface)
    }

    fn recompute_bounds(&mut self) {
        let (max_w, max_h) =
            self.views
                .iter_mut()
                .fold((0u32, 0u32), |(max_w, max_h), (_, view)| {
                    let max_w = u32::max(max_w, (view.position.x + view.size.width as i32) as u32);
                    let max_h = u32::max(max_h, (view.position.y + view.size.height as i32) as u32);
                    (max_w, max_h)
                });
        self.current_bounds = dpi::PhysicalSize::new(max_w, max_h);
        unsafe { self.on_bounds_changed() }.expect("updating bounds should succeed");
    }

    unsafe fn on_bounds_changed(&mut self) -> Result<(), windows::core::Error> {
        SetWindowPos(
            self.window,
            HWND(null_mut()),
            0,
            0,
            self.current_bounds.width as i32,
            self.current_bounds.height as i32,
            SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE,
        )?;

        self.bg_transform
            .SetScaleX2(self.current_bounds.width as f32)?;
        self.bg_transform
            .SetScaleY2(self.current_bounds.height as f32)?;
        self.dcomp_device.Commit()?;

        Ok(())
    }
}

impl Drop for WindowsCompositor {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.window);
        }
    }
}
impl NativeCompositor for WindowsCompositor {
    fn new_view(&mut self, attrs: super::NativeViewAttributes) -> Box<dyn super::NativeView> {
        let view = unsafe {
            self.new_view_inner(attrs)
                .expect("view creation should succeed")
        };
        let window = view.window;
        let view_key = self.views.insert(view);

        self.recompute_bounds();

        Box::new(WindowsViewHandle {
            view_key,
            display_state: self.display_state.clone(),
            window,
        })
    }

    fn delete_view(&mut self, view_key: super::NativeViewKey) {
        if self.views.remove(view_key).is_none() {
            panic!("delete_view should be called with a valid key")
        };

        self.recompute_bounds();
    }

    fn set_view_bounds(
        &mut self,
        view_key: super::NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {
        let view = self
            .views
            .get_mut(view_key)
            .expect("set_view_bounds requires a valid key");

        if position.is_none() && size.is_none() {
            return;
        }

        if let Some(position) = position {
            unsafe { view.set_position(position) };
        }
        if let Some(size) = size {
            unsafe { view.set_size(size) };
        }

        self.recompute_bounds();
    }

    fn restack_view(&mut self, view_key: super::NativeViewKey, stack_order: super::StackOrder) {
        log::warn!("Restacking not implemented on Windows yet");
    }

    fn total_bounds(&self) -> dpi::PhysicalSize<u32> {
        self.current_bounds
    }

    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        unsafe {
            SetWindowPos(
                self.window,
                HWND(null_mut()),
                position.x,
                position.y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .expect("msg")
        };
    }

    fn set_mapped(&mut self, mapped: bool) {
        unsafe {
            let sw_command = match mapped {
                true => SW_SHOWNOACTIVATE,
                false => SW_HIDE,
            };
            let _ = ShowWindow(self.window, sw_command);
        }
    }

    // fn scale_factor(&self) -> Option<f64> {
    //     Some(1.0)
    // }
}
impl WindowsCompositor {
    unsafe fn new_view_inner(
        &mut self,
        attrs: super::NativeViewAttributes,
    ) -> Result<WindowsView, windows::core::Error> {
        let size: dpi::PhysicalSize<u32> = attrs.surface_size;
        let position: dpi::PhysicalPosition<i32> = attrs.position;
        let transparent: bool = attrs.transparent;

        // Shoutouts to random other project
        // https://github.com/orca-app/orca/blob/e7938a832112b9b37fa750054e493823f8550a6d/src/graphics/win32_surface.c#L95

        let window = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            state::COMP_WINDOW_CLASS,
            w!("compositor view"),
            WS_POPUP,
            0,
            0,
            size.width as i32,
            size.height as i32,
            self.window,
            HMENU(null_mut()),
            self.display_state.hinstance,
            None,
        )?;

        // enable transparency using DwmEnableBlurBehindWindow
        if transparent {
            let region = CreateRectRgn(0, 0, -1, -1);

            let blur_behind = DWM_BLURBEHIND {
                dwFlags: DWM_BB_ENABLE | DWM_BB_BLURREGION,
                fEnable: BOOL::from(true),
                hRgnBlur: region,
                ..Default::default()
            };

            DwmEnableBlurBehindWindow(window, &blur_behind)?
        }

        // Cloak and show the window
        // This enables us to draw to it, but it won't be displayed
        {
            let cloaked = BOOL::from(true);
            DwmSetWindowAttribute(
                window,
                DWMWA_CLOAK,
                &cloaked as *const BOOL as *const _,
                std::mem::size_of_val(&cloaked) as u32,
            )?;
            let _ = ShowWindow(window, SW_SHOWNOACTIVATE);
        }

        // Setup a visual that can be passed to DComp
        let visual = self.dcomp_device.CreateVisual()?;
        let surface = self.dcomp_device.CreateSurfaceFromHwnd(window)?;

        visual.SetContent(&surface)?;
        visual.SetOffsetX2(position.x as f32)?;
        visual.SetOffsetY2(position.y as f32)?;

        // Add it to the existing tree (TODO: add Z-order support)
        self.root_visual.AddVisual(&visual, true, &self.bg_visual)?;
        self.dcomp_device.Commit()?;

        Ok(WindowsView {
            window,
            dcomp_device: self.dcomp_device.clone(),
            visual,
            surface,
            position,
            size,
        })
    }
}

impl WindowsView {
    unsafe fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        self.position = position;
        self.visual
            .SetOffsetX2(position.x as f32)
            .expect("IDCompositionVisual::SetOffsetX should succeed");
        self.visual
            .SetOffsetY2(position.y as f32)
            .expect("IDCompositionVisual::SetOffsetY should succeed");

        self.dcomp_device
            .Commit()
            .expect("IDCompositionDevice::Commit should succeed");
    }

    unsafe fn set_size(&mut self, size: dpi::PhysicalSize<u32>) {
        self.size = size;

        SetWindowPos(
            self.window,
            HWND(null_mut()),
            0,
            0,
            size.width as i32,
            size.height as i32,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREPOSITION,
        )
        .expect("SetWindowPos should succeed");
    }
}
impl Drop for WindowsView {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.window);
        }
    }
}

// SAFETY: HWND is safe to pass between threads for the sake of creating graphics
// objects. The handle is valid as long as the key is not deleted.
unsafe impl Send for WindowsViewHandle {}
unsafe impl Sync for WindowsViewHandle {}

impl NativeView for WindowsViewHandle {
    fn key(&self) -> NativeViewKey {
        self.view_key
    }

    fn display_handle_src(&self) -> &dyn raw_window_handle::HasDisplayHandle {
        self
    }

    fn window_handle_src(&self) -> &dyn raw_window_handle::HasWindowHandle {
        self
    }

    fn gl_api_preference(&self) -> DisplayApiPreference {
        DisplayApiPreference::Wgl(Some(self.window_handle().unwrap().as_raw()))
    }
}

impl HasDisplayHandle for WindowsViewHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::windows())
    }
}

impl HasWindowHandle for WindowsViewHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let mut raw_handle =
            Win32WindowHandle::new(NonZero::new(self.window.0 as usize as isize).unwrap());
        raw_handle.hinstance = NonZero::new(self.display_state.hinstance.0 as usize as isize);

        unsafe { Ok(WindowHandle::borrow_raw(raw_handle.into())) }
    }
}
