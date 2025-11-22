//! Bonjour implementation for cross-platform service.

use super::service_ref::{ManagedDNSServiceRef, RegisterServiceParams};
use super::{bonjour_util, constants};
use crate::ffi::c_str::{self, AsCChars};
use crate::ffi::{AsRaw, FromRaw, UnwrapOrNull};
use crate::prelude::*;
use crate::{
    EventLoop, NetworkInterface, Result, ServiceRegisteredCallback, ServiceRegistration,
    ServiceType, TxtRecord,
};
use bonjour_sys::{DNSServiceErrorType, DNSServiceFlags, DNSServiceRef};
use libc::{c_char, c_void};
use std::any::Any;
use std::ffi::CString;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct BonjourMdnsService {
    service: Arc<Mutex<ManagedDNSServiceRef>>,
    kind: CString,
    port: u16,
    name: Option<CString>,
    domain: Option<CString>,
    host: Option<CString>,
    interface_index: u32,
    txt_record: Option<TxtRecord>,
    context: Box<BonjourServiceContext>,
}

unsafe impl Send for BonjourMdnsService {}
unsafe impl Sync for BonjourMdnsService {}

impl TMdnsService for BonjourMdnsService {
    fn new(service_type: ServiceType, port: u16) -> Self {
        Self {
            service: Arc::default(),
            kind: bonjour_util::format_regtype(&service_type),
            port,
            name: None,
            domain: None,
            host: None,
            interface_index: constants::BONJOUR_IF_UNSPEC,
            txt_record: None,
            context: Box::default(),
        }
    }

    /// Sets the name to register this service under. If no name is set, Bonjour will
    /// automatically assign one (usually to the name of the machine).
    fn set_name(&mut self, name: &str) {
        self.name = Some(c_string!(name));
    }

    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(c_str::to_str)
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = bonjour_util::interface_index(interface);
    }

    fn network_interface(&self) -> NetworkInterface {
        bonjour_util::interface_from_index(self.interface_index)
    }

    fn set_domain(&mut self, domain: &str) {
        self.domain = Some(c_string!(domain));
    }

    fn domain(&self) -> Option<&str> {
        self.domain.as_ref().map(c_str::to_str)
    }

    fn set_host(&mut self, host: &str) {
        self.host = Some(c_string!(host));
    }

    fn host(&self) -> Option<&str> {
        self.host.as_ref().map(c_str::to_str)
    }

    fn set_txt_record(&mut self, txt_record: TxtRecord) {
        self.txt_record = Some(txt_record);
    }

    fn txt_record(&self) -> Option<&TxtRecord> {
        self.txt_record.as_ref()
    }

    fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>) {
        self.context.registered_callback = Some(registered_callback);
    }

    fn set_context(&mut self, context: Box<dyn Any + Send + Sync>) {
        self.context.user_context = Some(Arc::from(context));
    }

    fn context(&self) -> Option<&(dyn Any + Send + Sync)> {
        self.context.user_context.as_ref().map(|c| c.as_ref())
    }

    fn register(&mut self) -> Result<EventLoop> {
        debug!("Registering service: {:?}", self);

        let txt_len = self
            .txt_record
            .as_ref()
            .map(|t| unsafe { t.inner().get_length() })
            .unwrap_or(0);

        let txt_record = self
            .txt_record
            .as_ref()
            .map(|t| unsafe { t.inner().get_bytes_ptr() })
            .unwrap_or_null();

        let mut service_lock = self
            .service
            .lock()
            .expect("should be able to obtain lock on service");

        let register_params = RegisterServiceParams::builder()
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
            .context(self.context.as_raw())
            .build()?;

        unsafe { service_lock.register_service(register_params)? };

        Ok(EventLoop::new(self.service.clone()))
    }
}

#[derive(Default, FromRaw, AsRaw)]
struct BonjourServiceContext {
    registered_callback: Option<Box<ServiceRegisteredCallback>>,
    user_context: Option<Arc<dyn Any + Send + Sync>>,
}

// Necessary for BonjourMdnsService, cant be `derive`d because of registered_callback
impl std::fmt::Debug for BonjourServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BonjourServiceContext")
            .field("user_context", &self.user_context)
            .finish()
    }
}

unsafe impl Send for BonjourServiceContext {}
unsafe impl Sync for BonjourServiceContext {}

impl BonjourServiceContext {
    fn invoke_callback(&self, result: Result<ServiceRegistration>) {
        if let Some(f) = &self.registered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke callback but none was set");
        }
    }
}

unsafe extern "system" fn register_callback(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    error: DNSServiceErrorType,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    context: *mut c_void,
) {
    let context = unsafe { BonjourServiceContext::from_raw(context) };
    if let Err(e) = unsafe { handle_register(context, error, domain, name, regtype) } {
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

    let domain = bonjour_util::normalize_domain(unsafe { c_str::raw_to_str(domain) });
    let kind = bonjour_util::normalize_domain(unsafe { c_str::raw_to_str(regtype) });

    let result = ServiceRegistration::builder()
        .name(unsafe { c_str::copy_raw(name) })
        .service_type(bonjour_util::parse_regtype(&kind)?)
        .domain(domain)
        .build()
        .expect("could not build ServiceRegistration");

    context.invoke_callback(Ok(result));

    Ok(())
}
