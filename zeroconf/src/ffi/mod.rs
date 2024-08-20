//! Utilities related to FFI bindings

use libc::c_void;
use std::ptr;

pub(crate) mod c_str;

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

/// Helper trait to convert self to a raw `*mut c_void`
pub trait AsRaw {
    /// Converts self to a raw `*mut c_void` by cast.
    fn as_raw(&mut self) -> *mut c_void {
        self as *mut _ as *mut c_void
    }
}

/// Helper trait to unwrap a type to a `*const T` or a null-pointer if not present.
pub trait UnwrapOrNull<T> {
    /// Unwraps this type to `*const T` or `ptr::null()` if not present.
    fn unwrap_or_null(&self) -> *const T;
}

impl<T> UnwrapOrNull<T> for Option<*const T> {
    fn unwrap_or_null(&self) -> *const T {
        self.unwrap_or_else(ptr::null)
    }
}

/// Helper trait to unwrap a type to a `*mut T` or a null-pointer if not present.
#[cfg(target_os = "linux")]
pub trait UnwrapMutOrNull<T> {
    /// Unwraps this type to `*mut T` or `ptr::null_mut()` if not present.
    fn unwrap_mut_or_null(&mut self) -> *mut T;
}

#[cfg(target_os = "linux")]
impl<T> UnwrapMutOrNull<T> for Option<*mut T> {
    fn unwrap_mut_or_null(&mut self) -> *mut T {
        self.unwrap_or_else(ptr::null_mut)
    }
}

#[cfg(target_vendor = "apple")]
pub(crate) mod bonjour {
    use crate::{Error, Result};
    use libc::{fd_set, suseconds_t, time_t, timeval};
    use std::time::Duration;
    use std::{mem, ptr};

    /// Performs a unix `select()` on the specified `sock_fd` and `timeout`. Returns the select result
    /// or `Err` if the result is negative.
    ///
    /// # Safety
    /// This function is unsafe because it directly interfaces with C-library system calls.
    pub unsafe fn read_select(sock_fd: i32, timeout: Duration) -> Result<u32> {
        let mut read_flags: fd_set = mem::zeroed();

        libc::FD_ZERO(&mut read_flags);
        libc::FD_SET(sock_fd, &mut read_flags);

        let tv_sec = timeout.as_secs() as time_t;
        let tv_usec = timeout.subsec_micros() as suseconds_t;
        let mut timeout = timeval { tv_sec, tv_usec };

        let result = libc::select(
            sock_fd + 1,
            &mut read_flags,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut timeout,
        );

        if result < 0 {
            Err(Error::SystemError {
                code: result,
                message: "select(): returned error status".to_string(),
            })
        } else {
            Ok(result as u32)
        }
    }
}

#[cfg(target_vendor = "pc")]
pub(crate) mod bonjour {
    use crate::Result;
    use bonjour_sys::{dnssd_sock_t, fd_set, select, timeval};
    #[cfg(target_vendor = "apple")]
    use std::mem;
    use std::ptr;
    use std::time::Duration;

    /// Performs a unix `select()` on the specified `sock_fd` and `timeout`. Returns the select result
    /// or `Err` if the result is negative.
    ///
    /// # Safety
    /// This function is unsafe because it directly interfaces with C-library system calls.
    pub unsafe fn read_select(sock_fd: dnssd_sock_t, timeout: Duration) -> Result<u32> {
        if timeout.as_secs() > i32::MAX as u64 {
            return Err(
                "Invalid timeout duration, as_secs() value exceeds ::libc::c_long. ".into(),
            );
        }

        let timeout: timeval = timeval {
            tv_sec: timeout.as_secs() as ::libc::c_long,
            tv_usec: timeout.subsec_micros() as ::libc::c_long,
        };

        let mut set: fd_set = fd_set {
            fd_count: 1,
            fd_array: [0; 64],
        };
        set.fd_array[0] = sock_fd;

        let result = select(0, &mut set, ptr::null_mut(), &mut set, &timeout);

        if result < 0 {
            Err("select(): returned error status".into())
        } else {
            Ok(result as u32)
        }
    }
}
