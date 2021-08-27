//! Bonjour implementation for cross-platform browser

use super::service_ref::{
    BrowseServicesParams, GetAddressInfoParams, ManagedDNSServiceRef, ServiceResolveParams,
};
use super::txt_record_ref::ManagedTXTRecordRef;
use super::{bonjour_util, constants};
use crate::browser::BrowseFuture;
use crate::ffi::{c_str, AsRaw, FromRaw};
use crate::prelude::*;
use crate::{EventLoop, NetworkInterface, Result, ServiceType, TxtRecord};
use crate::{ServiceDiscoveredCallback, ServiceDiscovery};
use bonjour_sys::{DNSServiceErrorType, DNSServiceFlags, DNSServiceRef};
use libc::{c_char, c_uchar, c_void, sockaddr_in};
use std::any::Any;
use std::ffi::CString;
use std::fmt::{self, Formatter};
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::ptr;
use std::str::FromStr;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;

#[derive(Debug)]
pub struct BonjourMdnsBrowser {
    event_loop: EventLoop,
    timeout: Duration,
    is_init: bool,
    kind: CString,
    interface_index: u32,
    context: *mut BonjourBrowserContext,
}

impl TMdnsBrowser for BonjourMdnsBrowser {
    fn new(service_type: ServiceType) -> Self {
        Self {
            event_loop: EventLoop::default(),
            timeout: Duration::from_secs(0),
            is_init: false,
            kind: c_string!(service_type.to_string()),
            interface_index: constants::BONJOUR_IF_UNSPEC,
            context: Box::into_raw(Box::default()),
        }
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = bonjour_util::interface_index(interface);
    }

    fn set_service_discovered_callback(
        &mut self,
        service_discovered_callback: Box<ServiceDiscoveredCallback>,
    ) {
        unsafe { (*self.context).service_discovered_callback = Some(service_discovered_callback) };
    }

    fn set_context(&mut self, context: Box<dyn Any>) {
        unsafe { (*self.context).user_context = Some(Arc::from(context)) };
    }

    fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    fn browse(&mut self) -> Result<&EventLoop> {
        debug!("Browsing services: {:?}", self);

        self.event_loop.service_mut().browse(
            BrowseServicesParams::builder()
                .flags(0)
                .interface_index(self.interface_index)
                .regtype(self.kind.as_ptr())
                .domain(ptr::null_mut())
                .callback(Some(browse_callback))
                .context(self.context as *mut c_void)
                .build()?,
        )?;

        self.is_init = true;

        Ok(&self.event_loop)
    }

    fn browse_async(&mut self) -> BrowseFuture {
        Box::pin(BonjourBrowseFuture::new(self))
    }
}

impl Drop for BonjourMdnsBrowser {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
    }
}

#[derive(new)]
struct BonjourBrowseFuture<'a> {
    browser: &'a mut BonjourMdnsBrowser,
}

impl<'a> Future for BonjourBrowseFuture<'a> {
    type Output = Result<ServiceDiscovery>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = ctx.waker();
        let browser = &mut self.browser;
        if let Some(result) = unsafe { (*browser.context).discovered_service.take() } {
            debug!("BonjourBrowseFuture::poll() result = {:?}", result);
            Poll::Ready(result)
        } else if browser.is_init {
            debug!("BonjourBrowseFuture::poll() polling");
            if let Err(error) = browser.event_loop.poll(browser.timeout) {
                return Poll::Ready(Err(error));
            }
            debug!("BonjourBrowseFuture::poll() POLLED");
            waker.wake_by_ref();
            Poll::Pending
        } else {
            debug!("BonjourBrowseFuture::poll() initializing");
            if let Err(error) = browser.browse() {
                return Poll::Ready(Err(error));
            }
            waker.wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Default, FromRaw, AsRaw)]
struct BonjourBrowserContext {
    discovered_service: Option<Result<ServiceDiscovery>>,
    service_discovered_callback: Option<Box<ServiceDiscoveredCallback>>,
    resolved_name: Option<String>,
    resolved_kind: Option<String>,
    resolved_domain: Option<String>,
    resolved_port: u16,
    resolved_txt: Option<TxtRecord>,
    user_context: Option<Arc<dyn Any>>,
}

impl BonjourBrowserContext {
    fn invoke_callback(&mut self, result: Result<ServiceDiscovery>) {
        debug!(
            "BonjourBrowserContext::invoke_callback() result = {:?}",
            result
        );
        self.discovered_service = Some(result.clone());
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
            .field("discovered_service", &self.discovered_service)
            .field(
                "service_discovered_callback",
                &self
                    .service_discovered_callback
                    .as_ref()
                    .map(|_| "Some(Box<ServiceDiscoveredCallback>)")
                    .unwrap_or("None"),
            )
            .field("resolved_name", &self.resolved_name)
            .field("resolved_kind", &self.resolved_kind)
            .field("resolved_domain", &self.resolved_domain)
            .field("resolved_port", &self.resolved_port)
            .field("resolved_txt", &self.resolved_txt)
            .field("user_context", &self.user_context)
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
            .protocol(1)
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
    let ip = {
        let address = address as *const sockaddr_in;
        assert_not_null!(address);
        let s_addr = (*address).sin_addr.s_addr.to_le_bytes();
        IpAddr::from(s_addr).to_string()
    };

    let hostname = c_str::copy_raw(hostname);
    let domain = bonjour_util::normalize_domain(&ctx.resolved_domain.take().unwrap());
    let kind = bonjour_util::normalize_domain(&ctx.resolved_kind.take().unwrap());

    let result = ServiceDiscovery::builder()
        .name(ctx.resolved_name.take().unwrap())
        .service_type(ServiceType::from_str(&kind)?)
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
