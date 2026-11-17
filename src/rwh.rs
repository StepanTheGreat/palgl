use std::{ffi::{c_int, c_ulong, c_void}, num::{NonZeroIsize, NonZeroU32}, ptr::NonNull};

use wgpu::rwh::{RawDisplayHandle, RawWindowHandle};

/// An abstract window handle for various platforms
#[repr(transparent)]
pub struct WindowHandle(Box<RawWindowHandle>);

impl WindowHandle {
    pub fn handle(&self) -> &RawWindowHandle {
        &self.0
    }
}

/// An abstract display handle for various platforms
#[repr(transparent)]
pub struct DisplayHandle(Box<RawDisplayHandle>);

impl DisplayHandle {
    pub fn handle(&self) -> &RawDisplayHandle {
        &self.0
    }
}

// Everything that can get converted into a raw window handle - can also get converted into our abstract handle
impl<T: Into<RawWindowHandle>> From<T> for WindowHandle {
    fn from(value: T) -> Self {
        WindowHandle(Box::new(value.into()))
    }
} 

// Everything that can get converted into a raw window handle - can also get converted into our abstract handle
impl<T: Into<RawDisplayHandle>> From<T> for DisplayHandle {
    fn from(value: T) -> Self {
        DisplayHandle(Box::new(value.into()))
    }
} 

/// # Safety
/// It is assumed that the Win32 handle belongs to the current thread. This is necessary for the handle to be considered "valid" in all cases.
pub fn window_handle_win32(hwnd: NonZeroIsize) -> WindowHandle {
    wgpu::rwh::Win32WindowHandle::new(hwnd).into()
}

pub fn window_handle_xlib(window: c_ulong) -> WindowHandle {
    wgpu::rwh::XlibWindowHandle::new(window).into()
}

pub fn window_handle_xcb(window: NonZeroU32) -> WindowHandle {
    wgpu::rwh::XcbWindowHandle::new(window).into()
}

pub fn window_handle_wayland(surface: NonNull<c_void>) -> WindowHandle {
    wgpu::rwh::WaylandWindowHandle::new(surface).into()
}

pub fn window_handle_android(a_native_window: NonNull<c_void>) -> WindowHandle {    
    wgpu::rwh::AndroidNdkWindowHandle::new(a_native_window).into()
}

pub fn window_handle_appkit(ns_view: NonNull<c_void>) -> WindowHandle {
    wgpu::rwh::AppKitWindowHandle::new(ns_view).into()
}

pub fn window_handle_uikit(ui_view: NonNull<c_void>) -> WindowHandle {
    wgpu::rwh::UiKitWindowHandle::new(ui_view).into()
}


pub fn display_handle_win32() -> DisplayHandle {
    wgpu::rwh::WindowsDisplayHandle::new().into()
}

pub fn display_handle_xlib(connection: *mut c_void, screen: c_int) -> DisplayHandle {
    wgpu::rwh::XcbDisplayHandle::new(NonNull::new(connection), screen).into()
}

pub fn display_handle_xcb(connection: *mut c_void, screen: c_int) -> DisplayHandle {
    wgpu::rwh::XcbDisplayHandle::new(NonNull::new(connection), screen).into()
}

pub fn display_handle_wayland(display: NonNull<c_void>) -> DisplayHandle {
    wgpu::rwh::WaylandDisplayHandle::new(display).into()
}

pub fn display_handle_android() -> DisplayHandle {    
    wgpu::rwh::AndroidDisplayHandle::new().into()
}

pub fn display_handle_appkit() -> DisplayHandle {
    wgpu::rwh::AppKitDisplayHandle::new().into()
}

pub fn display_handle_uikit() -> DisplayHandle {
    wgpu::rwh::UiKitDisplayHandle::new().into()
}