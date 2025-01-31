//! Plugin interface extensions for connecting the frontend.

use std::{
    ffi::{c_char, c_void, CString},
    ptr::null_mut,
};

/// Plugin interface provided by m64prs-compatible frontends. A plugin
/// may obtain this interface struct from the frontend by exposing the
/// following function:
/// ```c
/// void M64PRS_UseFrontendInterface(struct FrontendInterfaceFFI* ffi);
/// ```
///
/// The passed pointer acts as a `&'static FrontendInterfaceFFI`: it
/// will outlive the plugin in its entirety. It is also guaranteed to
/// be implemented in a thread-safe manner. This may result in some blocking
/// as functions await the main UI thread for data.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FrontendInterfaceFFI {
    /// Context to pass to callbacks.
    pub context: *mut c_void,
    /// Gets a string token from the frontend for the main window handle.
    /// Up to `out_size` characters will be written to the array at `out_data`,
    /// including the terminating null character. `out_data` can also be set to
    /// null, in which case the function returns the expected length of the string
    /// (including the terminating null character).
    /// # Return value
    /// The number of characters written to `out_data`, or the number of characters
    /// expected (including the terminating null character) if `out_data` is a null pointer.
    ///
    /// # Notes
    /// - **Windows**: The string is the `HWND` encoded in hexadecimal.
    /// - **X11**: The string is the X11 window ID encoded in hexadecimal.
    /// - **Wayland**: The string is a handle from `xdg-foreign`.
    pub foreign_handle:
        unsafe extern "C" fn(context: *mut c_void, out_data: *mut c_char, out_size: usize) -> usize,
}

impl FrontendInterfaceFFI {
    pub fn foreign_handle(&self) -> CString {
        unsafe {
            let len = (self.foreign_handle)(self.context, null_mut(), 0);
            let mut data_array = vec![0u8; len];

            let copied_len =
                (self.foreign_handle)(self.context, data_array.as_mut_ptr() as *mut c_char, len);
            if copied_len != len {
                panic!("failed to copy the entire handle!");
            }
            CString::from_vec_unchecked(data_array)
        }
    }
}

unsafe impl Send for FrontendInterfaceFFI {}
unsafe impl Sync for FrontendInterfaceFFI {}

// FUNCTION PROTOTYPES
// ========================

#[allow(non_camel_case_types)]
pub type ptr_M64PRS_UseFrontendHandle =
    Option<unsafe extern "C" fn(ffi: *const FrontendInterfaceFFI)>;
