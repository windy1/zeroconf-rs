//! Low level interface for interacting with `DNSserviceRef`

use crate::{bonjour::bonjour_util, Result};
use bonjour_sys::{
    dnssd_sock_t, DNSServiceBrowse, DNSServiceBrowseReply, DNSServiceFlags, DNSServiceGetAddrInfo,
    DNSServiceGetAddrInfoReply, DNSServiceProcessResult, DNSServiceProtocol, DNSServiceRef,
    DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister, DNSServiceRegisterReply,
    DNSServiceResolve, DNSServiceResolveReply,
};
use libc::{c_char, c_void};
use std::ptr;

/// Wraps the `DNSServiceRef` type from the raw Bonjour bindings.
///
/// This struct allocates a new `DNSServiceRef` when any of the delegate functions is invoked and
/// calls the Bonjour function responsible for freeing the client on `trait Drop`.
///
/// # Note
/// This wrapper is meant for one-off calls to underlying Bonjour functions. The behavior for
/// using an already initialized `DNSServiceRef` in one of these functions is undefined. Therefore,
/// it is preferable to only call one delegate function per-instance.
#[derive(Debug)]
pub struct ManagedDNSServiceRef(DNSServiceRef);

impl ManagedDNSServiceRef {
    /// Constructs a new `ManagedDNSServiceRef`.
    pub fn new() -> Self {
        Self(ptr::null_mut())
    }

    /// Delegate function for [`DNSServiceRegister`].
    ///
    /// [`DNSServiceRegister`]: https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister?language=objc
    ///
    /// # Safety
    /// This function is unsafe because it calls a C function.
    pub unsafe fn register_service(
        &mut self,
        RegisterServiceParams {
            flags,
            interface_index,
            name,
            regtype,
            domain,
            host,
            port,
            txt_len,
            txt_record,
            callback,
            context,
        }: RegisterServiceParams,
    ) -> Result<()> {
        bonjour_util::sys_exec(
            || unsafe {
                DNSServiceRegister(
                    &mut self.0 as *mut DNSServiceRef,
                    flags,
                    interface_index,
                    name,
                    regtype,
                    domain,
                    host,
                    port.to_be(),
                    txt_len,
                    txt_record,
                    callback,
                    context,
                )
            },
            "could not register service",
        )
    }

    /// Delegate function for [`DNSServiceBrowse`].
    ///
    /// [`DNSServiceBrowse`]: https://developer.apple.com/documentation/dnssd/1804742-dnsservicebrowse?language=objc
    ///
    /// # Safety
    /// This function is unsafe because it calls a C function.
    pub unsafe fn browse_services(
        &mut self,
        BrowseServicesParams {
            flags,
            interface_index,
            regtype,
            domain,
            callback,
            context,
        }: BrowseServicesParams,
    ) -> Result<()> {
        bonjour_util::sys_exec(
            || unsafe {
                DNSServiceBrowse(
                    &mut self.0 as *mut DNSServiceRef,
                    flags,
                    interface_index,
                    regtype,
                    domain,
                    callback,
                    context,
                )
            },
            "could not browse services",
        )
    }

    /// Delegate function fro [`DNSServiceResolve`].
    ///
    /// [`DNSServiceResolve`]: https://developer.apple.com/documentation/dnssd/1804744-dnsserviceresolve?language=objc
    ///
    /// # Safety
    /// This function is unsafe because it calls a C function.
    pub unsafe fn resolve_service(
        &mut self,
        ServiceResolveParams {
            flags,
            interface_index,
            name,
            regtype,
            domain,
            callback,
            context,
        }: ServiceResolveParams,
    ) -> Result<()> {
        bonjour_util::sys_exec(
            || unsafe {
                DNSServiceResolve(
                    &mut self.0 as *mut DNSServiceRef,
                    flags,
                    interface_index,
                    name,
                    regtype,
                    domain,
                    callback,
                    context,
                )
            },
            "DNSServiceResolve() reported error",
        )?;

        unsafe { self.process_result() }
    }

    /// Delegate function for [`DNSServiceGetAddrInfo`].
    ///
    /// [`DNSServiceGetAddrInfo`]: https://developer.apple.com/documentation/dnssd/1804700-dnsservicegetaddrinfo?language=objc
    ///
    /// # Safety
    /// This function is unsafe because it calls a C function.
    pub unsafe fn get_address_info(
        &mut self,
        GetAddressInfoParams {
            flags,
            interface_index,
            protocol,
            hostname,
            callback,
            context,
        }: GetAddressInfoParams,
    ) -> Result<()> {
        bonjour_util::sys_exec(
            || unsafe {
                DNSServiceGetAddrInfo(
                    &mut self.0 as *mut DNSServiceRef,
                    flags,
                    interface_index,
                    protocol,
                    hostname,
                    callback,
                    context,
                )
            },
            "DNSServiceGetAddrInfo() reported error",
        )?;

        unsafe { self.process_result() }
    }

    /// Delegate function for [`DNSServiceProcessResult`].
    ///
    /// [`DNSServiceProcessResult`]: https://developer.apple.com/documentation/dnssd/1804696-dnsserviceprocessresult?language=objc
    ///
    /// # Safety
    /// This function is unsafe because it calls a C function.
    pub unsafe fn process_result(&self) -> Result<()> {
        bonjour_util::sys_exec(
            || unsafe { DNSServiceProcessResult(self.0) },
            "could not process service result",
        )
    }

    /// Delegate function for [`DNSServiceRefSockFD`].
    ///
    /// [`DNSServiceRefSockFD`]: https://developer.apple.com/documentation/dnssd/1804698-dnsservicerefsockfd?language=objc
    ///
    /// # Safety
    /// This function is unsafe because it calls a C function.
    pub unsafe fn sock_fd(&self) -> dnssd_sock_t {
        unsafe { DNSServiceRefSockFD(self.0) }
    }
}

impl Default for ManagedDNSServiceRef {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ManagedDNSServiceRef {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                DNSServiceRefDeallocate(self.0);
            }
        }
    }
}

unsafe impl Send for ManagedDNSServiceRef {}

/// Holds parameters for `ManagedDNSServiceRef::register_service()`.
#[derive(Builder, BuilderDelegate)]
pub struct RegisterServiceParams {
    flags: DNSServiceFlags,
    interface_index: u32,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    host: *const c_char,
    port: u16,
    txt_len: u16,
    txt_record: *const c_void,
    callback: DNSServiceRegisterReply,
    context: *mut c_void,
}

/// Holds parameters for `ManagedDNSServiceRef::browse_services()`.
#[derive(Builder, BuilderDelegate)]
pub struct BrowseServicesParams {
    flags: DNSServiceFlags,
    interface_index: u32,
    regtype: *const c_char,
    domain: *const c_char,
    callback: DNSServiceBrowseReply,
    context: *mut c_void,
}

/// Holds parameters for `ManagedDNSServiceRef::resolve_service()`.
#[derive(Builder, BuilderDelegate)]
pub struct ServiceResolveParams {
    flags: DNSServiceFlags,
    interface_index: u32,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    callback: DNSServiceResolveReply,
    context: *mut c_void,
}

/// Holds parameters for `ManagedDNSServiceRef::get_address_info()`.
#[derive(Builder, BuilderDelegate)]
pub struct GetAddressInfoParams {
    flags: DNSServiceFlags,
    interface_index: u32,
    protocol: DNSServiceProtocol,
    hostname: *const c_char,
    callback: DNSServiceGetAddrInfoReply,
    context: *mut c_void,
}
