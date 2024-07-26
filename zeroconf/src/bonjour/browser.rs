//! Bonjour implementation for cross-platform browser

use super::service_ref::{
    BrowseServicesParams, GetAddressInfoParams, ManagedDNSServiceRef, ServiceResolveParams,
};
use super::txt_record_ref::ManagedTXTRecordRef;
use super::{bonjour_util, constants};
use crate::ffi::{c_str, AsRaw, FromRaw};
use crate::prelude::*;
use crate::{EventLoop, NetworkInterface, Result, ServiceType, TxtRecord};
use crate::{ServiceDiscoveredCallback, ServiceDiscovery};
#[cfg(target_vendor = "pc")]
use bonjour_sys::sockaddr_in;
use bonjour_sys::{DNSServiceErrorType, DNSServiceFlags, DNSServiceRef};
#[cfg(target_vendor = "apple")]
use libc::sockaddr_in;
use libc::{c_char, c_uchar, c_void};
use std::any::Any;
use std::cell::RefCell;
use std::ffi::CString;
use std::fmt::{self, Formatter};
use std::net::IpAddr;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
pub struct BonjourMdnsBrowser {
    service: Rc<RefCell<ManagedDNSServiceRef>>,
    kind: CString,
    interface_index: u32,
    context: Box<BonjourBrowserContext>,
}

impl TMdnsBrowser for BonjourMdnsBrowser {
    fn new(service_type: ServiceType) -> Self {
        Self {
            service: Rc::default(),
            kind: bonjour_util::format_regtype(&service_type),
            interface_index: constants::BONJOUR_IF_UNSPEC,
            context: Box::default(),
        }
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = bonjour_util::interface_index(interface);
    }

    fn network_interface(&self) -> NetworkInterface {
        bonjour_util::interface_from_index(self.interface_index)
    }

    fn set_service_discovered_callback(
        &mut self,
        service_discovered_callback: Box<ServiceDiscoveredCallback>,
    ) {
        self.context.service_discovered_callback = Some(service_discovered_callback);
    }

    fn set_context(&mut self, context: Box<dyn Any>) {
        self.context.user_context = Some(Arc::from(context));
    }

    fn context(&self) -> Option<&dyn Any> {
        self.context.user_context.as_ref().map(|c| c.as_ref())
    }

    fn browse_services(&mut self) -> Result<EventLoop> {
        debug!("Browsing services: {:?}", self);

        self.service.borrow_mut().browse_services(
            BrowseServicesParams::builder()
                .flags(0)
                .interface_index(self.interface_index)
                .regtype(self.kind.as_ptr())
                .domain(ptr::null_mut())
                .callback(Some(browse_callback))
                .context(self.context.as_raw())
                .build()?,
        )?;

        Ok(EventLoop::new(self.service.clone()))
    }
}

#[derive(Default, FromRaw, AsRaw)]
struct BonjourBrowserContext {
    service_discovered_callback: Option<Box<ServiceDiscoveredCallback>>,
    resolved_name: Option<String>,
    resolved_kind: Option<String>,
    resolved_domain: Option<String>,
    resolved_port: u16,
    resolved_txt: Option<TxtRecord>,
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

unsafe extern "system" fn browse_callback(
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

unsafe extern "system" fn resolve_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    interface_index: u32,
    error: DNSServiceErrorType,
    _fullname: *const c_char,
    host_target: *const c_char,
    port: u16,
    txt_len: u16,
    txt_record: *const c_uchar,
    context: *mut c_void,
) {
    let ctx = BonjourBrowserContext::from_raw(context);

    let result = handle_resolve(
        ctx,
        error,
        port,
        interface_index,
        host_target,
        txt_len,
        txt_record,
    );

    if let Err(e) = result {
        ctx.invoke_callback(Err(e));
    }
}

unsafe fn handle_resolve(
    ctx: &mut BonjourBrowserContext,
    error: DNSServiceErrorType,
    port: u16,
    interface_index: u32,
    host_target: *const c_char,
    txt_len: u16,
    txt_record: *const c_uchar,
) -> Result<()> {
    if error != 0 {
        return Err(format!("error reported by resolve_callback: (code: {})", error).into());
    }

    ctx.resolved_port = port;

    ctx.resolved_txt = if txt_len > 1 {
        Some(TxtRecord::from(ManagedTXTRecordRef::clone_raw(
            txt_record, txt_len,
        )?))
    } else {
        None
    };

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

unsafe extern "system" fn get_address_info_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    _interface_index: u32,
    error: DNSServiceErrorType,
    hostname: *const c_char,
    address: *const bonjour_sys::sockaddr,
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
    address: *const bonjour_sys::sockaddr,
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

    // on macOS the bytes are swapped for the port
    let port: u16 = ctx.resolved_port.to_be();

    // on macOS the bytes are swapped for the ip
    #[cfg(target_vendor = "apple")]
    let ip = {
        let address = address as *const sockaddr_in;
        assert_not_null!(address);
        let s_addr = (*address).sin_addr.s_addr.to_le_bytes();
        IpAddr::from(s_addr).to_string()
    };

    #[cfg(target_vendor = "pc")]
    let ip = {
        let address = address as *const sockaddr_in;
        assert_not_null!(address);
        let s_un = (*address).sin_addr.S_un.S_un_b;
        let s_addr = [s_un.s_b1, s_un.s_b2, s_un.s_b3, s_un.s_b4];
        IpAddr::from(s_addr).to_string()
    };

    let hostname = c_str::copy_raw(hostname);

    let domain = bonjour_util::normalize_domain(
        &ctx.resolved_domain
            .take()
            .ok_or("could not get domain from BonjourBrowserContext")?,
    );

    let kind = bonjour_util::normalize_domain(
        &ctx.resolved_kind
            .take()
            .ok_or("could not get kind from BonjourBrowserContext")?,
    );

    let name = ctx
        .resolved_name
        .take()
        .ok_or("could not get name from BonjourBrowserContext")?;

    let result = ServiceDiscovery::builder()
        .name(name)
        .service_type(bonjour_util::parse_regtype(&kind)?)
        .domain(domain)
        .host_name(hostname)
        .address(ip)
        .port(port)
        .txt(ctx.resolved_txt.take())
        .build()
        .expect("could not build ServiceResolution");

    ctx.invoke_callback(Ok(result));

    Ok(())
}
