use std::ptr::null_mut;

use windows::{
    core::Interface,
    Win32::{
        Foundation::{HMODULE, HWND},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_UNKNOWN,
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
            },
            DirectComposition::{
                DCompositionCreateDevice, IDCompositionDevice, IDCompositionSurface,
                IDCompositionTarget, IDCompositionVisual,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_STRAIGHT, DXGI_FORMAT_B8G8R8A8_UNORM}, CreateDXGIFactory2, IDXGIAdapter, IDXGIDevice,
                IDXGIDevice1, IDXGIFactory, IDXGIFactory2, DXGI_CREATE_FACTORY_FLAGS,
            },
            Gdi::HDC,
        },
    },
};

pub(super) struct DCompDevice {
    dx_factory: IDXGIFactory2,
    dx_adapter: IDXGIAdapter,

    d3d_device: ID3D11Device,
    dx_device: IDXGIDevice,

    dcomp_device: IDCompositionDevice,
    dcomp_target: IDCompositionTarget,
}

impl DCompDevice {
    unsafe fn new(window: HWND) -> Result<Self, windows::core::Error> {
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

        Ok(DCompDevice {
            dx_factory,
            dx_adapter,
            d3d_device,
            dx_device,
            dcomp_device,
            dcomp_target,
        })
    }
}
