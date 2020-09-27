use super::service_ref::{
    BrowseServicesParams, GetAddressInfoParams, ManagedDNSServiceRef, ServiceResolveParams,
};
use super::{compat, constants};
use crate::builder::BuilderDelegate;
use crate::ffi::{self, c_str, AsRaw, FromRaw};
use crate::{EventLoop, NetworkInterface, Result};
use crate::{ServiceDiscoveredCallback, ServiceDiscovery};
use bonjour_sys::{sockaddr, DNSServiceErrorType, DNSServiceFlags, DNSServiceRef};
use libc::{c_char, c_uchar, c_void, sockaddr_in};
use std::any::Any;
use std::ffi::CString;
use std::fmt::{self, Formatter};
use std::ptr;
use std::sync::{Arc, Mutex};

/// Interface for interacting with Bonjour's mDNS service browsing capabilities.
#[derive(Debug)]
pub struct BonjourMdnsBrowser {
    service: Arc<Mutex<ManagedDNSServiceRef>>,
    kind: CString,
    interface_index: u32,
    context: *mut BonjourBrowserContext,
}

impl BonjourMdnsBrowser {
    /// Creates a new `BonjourMdnsBrowser` that browses for the specified `kind`
    /// (e.g. `_http._tcp`).
    pub fn new(kind: &str) -> Self {
        Self {
            service: Arc::default(),
            kind: c_string!(kind),
            interface_index: constants::BONJOUR_IF_UNSPEC,
            context: Box::into_raw(Box::default()),
        }
    }

    /// Sets the network interface on which to browse for services on.
    ///
    /// Most applications will want to use the default value `NetworkInterface::Unspec` to browse
    /// on all available interfaces.
    pub fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = compat::interface_index(interface);
    }

    /// Sets the [`ServiceDiscoveredCallback`] that is invoked when the browser has discovered and
    /// resolved a service.
    ///
    /// [`ServiceDiscoveredCallback`]: ../type.ServiceDiscoveredCallback.html
    pub fn set_service_discovered_callback(
        &self,
        service_discovered_callback: Box<ServiceDiscoveredCallback>,
    ) {
        unsafe { (*self.context).service_discovered_callback = Some(service_discovered_callback) };
    }

    /// Sets the optional user context to pass through to the callback. This is useful if you need
    /// to share state between pre and post-callback. The context type must implement `Any`.
    pub fn set_context(&mut self, context: Box<dyn Any>) {
        unsafe { (*self.context).user_context = Some(Arc::from(context)) };
    }

    /// Starts the browser. Returns an [`EventLoop`] which can be called to keep the browser alive.
    pub fn browse_services(&mut self) -> Result<EventLoop> {
        debug!("Browsing services: {:?}", self);

        self.service.lock().unwrap().browse_services(
            BrowseServicesParams::builder()
                .flags(0)
                .interface_index(self.interface_index)
                .regtype(self.kind.as_ptr())
                .domain(ptr::null_mut())
                .callback(Some(browse_callback))
                .context(self.context as *mut c_void)
                .build()?,
        )?;

        Ok(EventLoop::new(self.service.clone()))
    }
}

impl Drop for BonjourMdnsBrowser {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
    }
}

#[derive(Default, FromRaw, AsRaw)]
struct BonjourBrowserContext {
    service_discovered_callback: Option<Box<ServiceDiscoveredCallback>>,
    resolved_name: Option<String>,
    resolved_kind: Option<String>,
    resolved_domain: Option<String>,
    resolved_port: u16,
    user_context: Option<Arc<dyn Any>>,
}

impl BonjourBrowserContext {
    fn invoke_callback(&self, result: Result<ServiceDiscovery>) {
        if let Some(f) = &self.service_discovered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke callback but none was set");
        }
    }
}

impl fmt::Debug for BonjourBrowserContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BonjourResolverContext")
            .field("resolved_name", &self.resolved_name)
            .field("resolved_kind", &self.resolved_kind)
            .field("resolved_domain", &self.resolved_domain)
            .field("resolved_port", &self.resolved_port)
            .finish()
    }
}

unsafe extern "C" fn browse_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    interface_index: u32,
    error: DNSServiceErrorType,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    context: *mut c_void,
) {
    let ctx = BonjourBrowserContext::from_raw(context);
    if let Err(e) = handle_browse(ctx, error, name, regtype, domain, interface_index) {
        ctx.invoke_callback(Err(e));
    }
}

unsafe fn handle_browse(
    ctx: &mut BonjourBrowserContext,
    error: DNSServiceErrorType,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    interface_index: u32,
) -> Result<()> {
    if error != 0 {
        return Err(format!("browse_callback() reported error (code: {})", error).into());
    }

    ctx.resolved_name = Some(c_str::copy_raw(name));
    ctx.resolved_kind = Some(c_str::copy_raw(regtype));
    ctx.resolved_domain = Some(c_str::copy_raw(domain));

    ManagedDNSServiceRef::default().resolve_service(
        ServiceResolveParams::builder()
            .flags(bonjour_sys::kDNSServiceFlagsForceMulticast)
            .interface_index(interface_index)
            .name(name)
            .regtype(regtype)
            .domain(domain)
            .callback(Some(resolve_callback))
            .context(ctx.as_raw())
            .build()?,
    )
}

unsafe extern "C" fn resolve_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    interface_index: u32,
    error: DNSServiceErrorType,
    _fullname: *const c_char,
    host_target: *const c_char,
    port: u16,
    _txt_len: u16,
    _txt_record: *const c_uchar,
    context: *mut c_void,
) {
    let ctx = BonjourBrowserContext::from_raw(context);
    if let Err(e) = handle_resolve(ctx, error, port, interface_index, host_target) {
        ctx.invoke_callback(Err(e));
    }
}

fn handle_resolve(
    ctx: &mut BonjourBrowserContext,
    error: DNSServiceErrorType,
    port: u16,
    interface_index: u32,
    host_target: *const c_char,
) -> Result<()> {
    if error != 0 {
        return Err(format!("error reported by resolve_callback: (code: {})", error).into());
    }

    ctx.resolved_port = port;

    ManagedDNSServiceRef::default().get_address_info(
        GetAddressInfoParams::builder()
            .flags(bonjour_sys::kDNSServiceFlagsForceMulticast)
            .interface_index(interface_index)
            .protocol(0)
            .hostname(host_target)
            .callback(Some(get_address_info_callback))
            .context(ctx.as_raw())
            .build()?,
    )
}

unsafe extern "C" fn get_address_info_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    _interface_index: u32,
    error: DNSServiceErrorType,
    hostname: *const c_char,
    address: *const sockaddr,
    _ttl: u32,
    context: *mut c_void,
) {
    let ctx = BonjourBrowserContext::from_raw(context);
    if let Err(e) = handle_get_address_info(ctx, error, address, hostname) {
        ctx.invoke_callback(Err(e));
    }
}

unsafe fn handle_get_address_info(
    ctx: &mut BonjourBrowserContext,
    error: DNSServiceErrorType,
    address: *const sockaddr,
    hostname: *const c_char,
) -> Result<()> {
    // this callback runs multiple times for some reason
    if ctx.resolved_name.is_none() {
        return Ok(());
    }

    if error != 0 {
        return Err(format!(
            "get_address_info_callback() reported error (code: {})",
            error
        )
        .into());
    }

    let ip = ffi::get_ip(address as *const sockaddr_in);
    let hostname = c_str::copy_raw(hostname);
    let domain = compat::normalize_domain(&ctx.resolved_domain.take().unwrap());

    let result = ServiceDiscovery::builder()
        .name(ctx.resolved_name.take().unwrap())
        .kind(ctx.resolved_kind.take().unwrap())
        .domain(domain)
        .host_name(hostname)
        .address(ip)
        .port(ctx.resolved_port)
        .build()
        .expect("could not build ServiceResolution");

    ctx.invoke_callback(Ok(result));

    Ok(())
}
