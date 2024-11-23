use std::{ffi::c_void, ptr::null_mut, sync::Arc};

use gdk::prelude::*;
use slotmap::DenseSlotMap;
use state::{DisplayState, Win32DisplayExt};
use windows::{
    core::Interface,
    Win32::{
        Foundation::{COLORREF, HMODULE, HWND, POINT},
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
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM},
                CreateDXGIFactory2, IDXGIAdapter, IDXGIDevice, IDXGIFactory2, IDXGISurface1,
                DXGI_CREATE_FACTORY_FLAGS,
            },
            Gdi::{self},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HMENU, WINDOW_EX_STYLE, WS_CHILD,
        },
    },
};

use super::{NativeCompositor, NativeViewKey};

mod state;
mod util;

struct WindowsCompositor {
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

struct WindowsView {}

struct WindowsViewHandle {}

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
            state::SUBSURFACE_WINDOW_CLASS,
            windows::core::w!("output window"),
            WS_CHILD,
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
        todo!()
    }

    fn delete_view(&mut self, view: super::NativeViewKey) {
        todo!()
    }

    fn set_view_bounds(
        &mut self,
        key: super::NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {
        todo!()
    }

    fn restack_view(&mut self, key: super::NativeViewKey, stack_order: super::StackOrder) {
        todo!()
    }

    fn total_bounds(&self) -> dpi::PhysicalSize<u32> {
        todo!()
    }

    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        todo!()
    }

    fn set_mapped(&mut self, mapped: bool) {
        todo!()
    }
}
