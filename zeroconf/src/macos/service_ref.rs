//! Rust friendly Bonjour wrappers/helpers

use crate::Result;
use bonjour_sys::{
    DNSServiceBrowse, DNSServiceBrowseReply, DNSServiceFlags, DNSServiceGetAddrInfo,
    DNSServiceGetAddrInfoReply, DNSServiceProcessResult, DNSServiceProtocol, DNSServiceRef,
    DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister, DNSServiceRegisterReply,
    DNSServiceResolve, DNSServiceResolveReply,
};
use libc::{c_char, c_void};
use std::ptr;

/// Wraps the `DNSServiceRef` type from the raw Bonjour bindings.
///
/// This struct allocates a new `DNSServiceRef` when any of the delgate functions is invoked and
/// calls the Bonjour function responsible for freeing the client on `trait Drop`.
///
/// # Note
/// This wrapper is meant for one-off calls to underlying Bonjour functions. The behaviour for
/// using an already initialized `DNSServiceRef` in one of these functions is undefined. Therefore,
/// it is preferable to only call one delegate function per-instance.
#[derive(Debug)]
pub struct ManagedDNSServiceRef(DNSServiceRef);

impl ManagedDNSServiceRef {
    /// Delegate function for [`DNSServiceRegister`].
    ///
    /// [`DNSServiceRegister`]: https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister?language=objc
    pub fn register_service(
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
        let err = unsafe {
            DNSServiceRegister(
                &mut self.0 as *mut DNSServiceRef,
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
            )
        };

        if err != 0 {
            return Err(format!("could not register service (code: {})", err).into());
        }

        Ok(())
    }

    /// Delegate function for [`DNSServiceBrowse`].
    ///
    /// [`DNSServiceBrowse`]: https://developer.apple.com/documentation/dnssd/1804742-dnsservicebrowse?language=objc
    pub fn browse_services(
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
        let err = unsafe {
            DNSServiceBrowse(
                &mut self.0 as *mut DNSServiceRef,
                flags,
                interface_index,
                regtype,
                domain,
                callback,
                context,
            )
        };

        if err != 0 {
            return Err(format!("could not browse services (code: {})", err).into());
        }

        Ok(())
    }

    /// Delegate function fro [`DNSServiceResolve`].
    ///
    /// [`DNSServiceResolve`]: https://developer.apple.com/documentation/dnssd/1804744-dnsserviceresolve?language=objc
    pub fn resolve_service(
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
        let error = unsafe {
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
        };

        if error != 0 {
            return Err(format!("DNSServiceResolve() reported error (code: {})", error).into());
        }

        self.process_result()
    }

    /// Delegate function for [`DNSServiceGetAddrInfo`].
    ///
    /// [`DNSServiceGetAddrInfo`]: https://developer.apple.com/documentation/dnssd/1804700-dnsservicegetaddrinfo?language=objc
    pub fn get_address_info(
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
        let err = unsafe {
            DNSServiceGetAddrInfo(
                &mut self.0 as *mut DNSServiceRef,
                flags,
                interface_index,
                protocol,
                hostname,
                callback,
                context,
            )
        };

        if err != 0 {
            return Err(format!("DNSServiceGetAddrInfo() reported error (code: {})", err).into());
        }

        self.process_result()
    }

    /// Delegate function for [`DNSServiceProcessResult`].
    ///
    /// [`DNSServiceProcessResult`]: https://developer.apple.com/documentation/dnssd/1804696-dnsserviceprocessresult?language=objc
    pub fn process_result(&self) -> Result<()> {
        let err = unsafe { DNSServiceProcessResult(self.0) };
        if err != 0 {
            Err(format!("could not process service result (code: {})", err).into())
        } else {
            Ok(())
        }
    }

    /// Delegate function for [`DNSServiceRefSockFD`].
    ///
    /// [`DNSServiceRefSockFD`]: https://developer.apple.com/documentation/dnssd/1804698-dnsservicerefsockfd?language=objc
    pub fn sock_fd(&self) -> i32 {
        unsafe { DNSServiceRefSockFD(self.0) }
    }
}

impl Default for ManagedDNSServiceRef {
    fn default() -> Self {
        Self(ptr::null_mut())
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
