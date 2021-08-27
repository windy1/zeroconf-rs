//! Bonjour implementation for cross-platform service.

use super::service_ref::RegisterServiceParams;
use super::{bonjour_util, constants};
use crate::ffi::c_str::{self, AsCChars};
use crate::ffi::{FromRaw, UnwrapOrNull};
use crate::prelude::*;
use crate::service::ServiceRegisterFuture;
use crate::{
    EventLoop, NetworkInterface, Result, ServiceRegisteredCallback, ServiceRegistration,
    ServiceType, TxtRecord,
};
use bonjour_sys::{DNSServiceErrorType, DNSServiceFlags, DNSServiceRef};
use libc::{c_char, c_void};
use std::any::Any;
use std::ffi::CString;
use std::fmt;
use std::fmt::Formatter;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;

#[derive(Debug)]
pub struct BonjourMdnsService {
    event_loop: EventLoop,
    is_init: bool,
    timeout: Duration,
    kind: CString,
    port: u16,
    name: Option<CString>,
    domain: Option<CString>,
    host: Option<CString>,
    interface_index: u32,
    txt_record: Option<TxtRecord>,
    context: *mut BonjourServiceContext,
}

impl TMdnsService for BonjourMdnsService {
    fn new(service_type: ServiceType, port: u16) -> Self {
        Self {
            event_loop: EventLoop::default(),
            is_init: false,
            timeout: Duration::from_secs(0),
            kind: c_string!(service_type.to_string()),
            port,
            name: None,
            domain: None,
            host: None,
            interface_index: constants::BONJOUR_IF_UNSPEC,
            txt_record: None,
            context: Box::into_raw(Box::default()),
        }
    }

    /// Sets the name to register this service under. If no name is set, Bonjour will
    /// automatically assign one (usually to the name of the machine).
    fn set_name(&mut self, name: &str) {
        self.name = Some(c_string!(name));
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = bonjour_util::interface_index(interface);
    }

    fn set_domain(&mut self, domain: &str) {
        self.domain = Some(c_string!(domain));
    }

    fn set_host(&mut self, host: &str) {
        self.host = Some(c_string!(host));
    }

    fn set_txt_record(&mut self, txt_record: TxtRecord) {
        self.txt_record = Some(txt_record);
    }

    fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>) {
        unsafe { (*self.context).registered_callback = Some(registered_callback) };
    }

    fn set_context(&mut self, context: Box<dyn Any>) {
        unsafe { (*self.context).user_context = Some(Arc::from(context)) };
    }

    fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    fn register(&mut self) -> Result<&EventLoop> {
        debug!("Registering service: {:?}", self);

        let txt_len = self
            .txt_record
            .as_ref()
            .map(|t| t.inner().get_length())
            .unwrap_or(0);

        let txt_record = self
            .txt_record
            .as_ref()
            .map(|t| t.inner().get_bytes_ptr())
            .unwrap_or_null();

        self.event_loop.service_mut().register(
            RegisterServiceParams::builder()
                .flags(constants::BONJOUR_RENAME_FLAGS)
                .interface_index(self.interface_index)
                .name(self.name.as_ref().as_c_chars().unwrap_or_null())
                .regtype(self.kind.as_ptr())
                .domain(self.domain.as_ref().as_c_chars().unwrap_or_null())
                .host(self.host.as_ref().as_c_chars().unwrap_or_null())
                .port(self.port)
                .txt_len(txt_len)
                .txt_record(txt_record)
                .callback(Some(register_callback))
                .context(self.context as *mut c_void)
                .build()?,
        )?;

        self.is_init = true;

        Ok(&self.event_loop)
    }

    fn register_async(&mut self) -> ServiceRegisterFuture {
        Box::pin(BonjourServiceRegisterFuture::new(self))
    }
}

impl Drop for BonjourMdnsService {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
    }
}

#[derive(new)]
struct BonjourServiceRegisterFuture<'a> {
    service: &'a mut BonjourMdnsService,
}

impl<'a> Future for BonjourServiceRegisterFuture<'a> {
    type Output = Result<ServiceRegistration>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = ctx.waker();
        let service = &mut self.service;
        if let Some(result) = unsafe { (*service.context).registration_result.take() } {
            Poll::Ready(result)
        } else if service.is_init {
            if let Err(error) = service.event_loop.poll(service.timeout) {
                return Poll::Ready(Err(error));
            }
            waker.wake_by_ref();
            Poll::Pending
        } else {
            if let Err(error) = service.register() {
                return Poll::Ready(Err(error));
            }
            waker.wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Default, FromRaw)]
struct BonjourServiceContext {
    registration_result: Option<Result<ServiceRegistration>>,
    registered_callback: Option<Box<ServiceRegisteredCallback>>,
    user_context: Option<Arc<dyn Any>>,
}

impl BonjourServiceContext {
    fn invoke_callback(&self, result: Result<ServiceRegistration>) {
        if let Some(f) = &self.registered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke service callback but none was set");
        }
    }
}

impl fmt::Debug for BonjourServiceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BonjourServiceContext")
            .field("registration_result", &self.registration_result)
            .field(
                "registered_callback",
                &self
                    .registered_callback
                    .as_ref()
                    .map(|_| "Some(Box<ServiceRegisteredCallback>)")
                    .unwrap_or("None"),
            )
            .field("user_context", &self.user_context)
            .finish()
    }
}

unsafe extern "C" fn register_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    error: DNSServiceErrorType,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    context: *mut c_void,
) {
    let context = BonjourServiceContext::from_raw(context);
    if let Err(e) = handle_register(context, error, domain, name, regtype) {
        context.invoke_callback(Err(e));
    }
}

unsafe fn handle_register(
    context: &BonjourServiceContext,
    error: DNSServiceErrorType,
    domain: *const c_char,
    name: *const c_char,
    regtype: *const c_char,
) -> Result<()> {
    if error != 0 {
        return Err(format!("register_callback() reported error (code: {0})", error).into());
    }

    let domain = bonjour_util::normalize_domain(c_str::raw_to_str(domain));
    let kind = bonjour_util::normalize_domain(c_str::raw_to_str(regtype));

    let result = ServiceRegistration::builder()
        .name(c_str::copy_raw(name))
        .service_type(ServiceType::from_str(&kind)?)
        .domain(domain)
        .build()
        .expect("could not build ServiceRegistration");

    context.invoke_callback(Ok(result));

    Ok(())
}
