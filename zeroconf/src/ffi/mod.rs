//! Utilities related to FFI bindings

use crate::Result;
use libc::{c_char, c_void, fd_set, in_addr, sockaddr_in, timeval};
use std::time::Duration;
use std::{mem, ptr};

pub mod c_str;

/// Helper trait to convert a raw `*mut c_void` to it's rust type
pub trait FromRaw<T> {
    /// Converts the specified `*mut c_void` to a `&'a mut T`.
    ///
    /// # Safety
    /// This function is unsafe due to the dereference of the specified raw pointer.
    unsafe fn from_raw<'a>(raw: *mut c_void) -> &'a mut T {
        assert_not_null!(raw);
        &mut *(raw as *mut T)
    }
}

/// Helper trait to convert and clone a raw `*mut c_void` to it's rust type
pub trait CloneRaw<T: FromRaw<T> + Clone> {
    /// Converts and clones the specified `*mut c_void` to a `Box<T>`.
    ///
    /// # Safety
    /// This function is unsafe due to a call to the unsafe function [`FromRaw::from_raw()`].
    ///
    /// [`FromRaw::from_raw()`]: trait.FromRaw.html#method.from_raw
    unsafe fn clone_raw(raw: *mut c_void) -> Box<T> {
        assert_not_null!(raw);
        Box::new(T::from_raw(raw).clone())
    }
}

/// Helper trait to convert self to a raw `*mut c_void`
pub trait AsRaw {
    /// Converts self to a raw `*mut c_void` by cast.
    fn as_raw(&mut self) -> *mut c_void {
        self as *mut _ as *mut c_void
    }
}

/// Performs a unix `select()` on the specified `sock_fd` and `timeout`. Returns the select result
/// or `Err` if the result is negative.
///
/// # Safety
/// This function is unsafe because it directly interfaces with C-library system calls.
pub unsafe fn read_select(sock_fd: i32, timeout: Duration) -> Result<u32> {
    let mut read_flags: fd_set = mem::zeroed();

    libc::FD_ZERO(&mut read_flags);
    libc::FD_SET(sock_fd, &mut read_flags);

    let tv_sec = timeout.as_secs() as i64;
    #[cfg(target_os = "macos")]
    let tv_usec = timeout.subsec_micros() as i32;
    #[cfg(target_os = "linux")]
    let tv_usec = timeout.subsec_micros() as i64;

    let mut timeout = timeval { tv_sec, tv_usec };

    let result = libc::select(
        sock_fd + 1,
        &mut read_flags,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut timeout,
    );

    if result < 0 {
        Err("select(): returned error status".into())
    } else {
        Ok(result as u32)
    }
}

/// Returns a human-readable address of the specified raw address
///
/// # Safety
/// This function is unsafe because of calls to C-library system calls
pub unsafe fn get_ip(address: *const sockaddr_in) -> String {
    let raw = inet_ntoa(&(*address).sin_addr as *const in_addr);
    String::from(c_str::raw_to_str(raw))
}

extern "C" {
    fn inet_ntoa(addr: *const in_addr) -> *const c_char;
}
