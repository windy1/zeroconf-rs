use super::service_ref::{ManagedDNSServiceRef, RegisterServiceParams};
use super::{bonjour_util, constants};
use crate::builder::BuilderDelegate;
use crate::ffi::c_str::{self, AsCChars};
use crate::ffi::{FromRaw, UnwrapOrNull};
use crate::{
    EventLoop, NetworkInterface, Result, ServiceRegisteredCallback, ServiceRegistration, TxtRecord,
};
use bonjour_sys::{DNSServiceErrorType, DNSServiceFlags, DNSServiceRef};
use libc::{c_char, c_void};
use std::any::Any;
use std::ffi::CString;
use std::sync::{Arc, Mutex};

/// Interface for interacting with Bonjour's mDNS service registration capabilities.
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
    context: *mut BonjourServiceContext,
}

impl BonjourMdnsService {
    /// Creates a new `BonjourMdnsService` with the specified `kind` (e.g. `_http._tcp`) and
    /// `port`.
    pub fn new(kind: &str, port: u16) -> Self {
        Self {
            service: Arc::default(),
            kind: c_string!(kind),
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
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(c_string!(name));
    }

    /// Sets the network interface to bind this service to.
    ///
    /// Most applications will want to use the default value `NetworkInterface::Unspec` to bind to
    /// all available interfaces.
    pub fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = bonjour_util::interface_index(interface);
    }

    /// Sets the domain on which to advertise the service.
    ///
    /// Most applications will want to use the default value of `ptr::null()` to register to the
    /// default domain.
    pub fn set_domain(&mut self, domain: &str) {
        self.domain = Some(c_string!(domain));
    }

    /// Sets the SRV target host name.
    ///
    /// Most applications will want to use the default value of `ptr::null()` to use the machine's
    // default host name.
    pub fn set_host(&mut self, host: &str) {
        self.host = Some(c_string!(host));
    }

    pub fn set_txt_record(&mut self, txt_record: TxtRecord) {
        self.txt_record = Some(txt_record);
    }

    /// Sets the [`ServiceRegisteredCallback`] that is invoked when the service has been
    /// registered.
    ///
    /// [`ServiceRegisteredCallback`]: ../type.ServiceRegisteredCallback.html
    pub fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>) {
        unsafe { (*self.context).registered_callback = Some(registered_callback) };
    }

    /// Sets the optional user context to pass through to the callback. This is useful if you need
    /// to share state between pre and post-callback. The context type must implement `Any`.
    pub fn set_context(&mut self, context: Box<dyn Any>) {
        unsafe { (*self.context).user_context = Some(Arc::from(context)) };
    }

    /// Registers and start's the service. Returns an `EventLoop` which can be called to keep
    /// the service alive.
    pub fn register(&mut self) -> Result<EventLoop> {
        debug!("Registering service: {:?}", self);

        let txt_record = self
            .txt_record
            .as_ref()
            .map(|t| t.as_ptr())
            .unwrap_or_null();

        self.service.lock().unwrap().register_service(
            RegisterServiceParams::builder()
                .flags(constants::BONJOUR_RENAME_FLAGS)
                .interface_index(self.interface_index)
                .name(self.name.as_ref().as_c_chars().unwrap_or_null())
                .regtype(self.kind.as_ptr())
                .domain(self.domain.as_ref().as_c_chars().unwrap_or_null())
                .host(self.host.as_ref().as_c_chars().unwrap_or_null())
                .port(self.port)
                .txt_len(self.txt_record.as_ref().map(|t| t.size()).unwrap_or(0))
                .txt_record(txt_record)
                .callback(Some(register_callback))
                .context(self.context as *mut c_void)
                .build()?,
        )?;

        Ok(EventLoop::new(self.service.clone()))
    }
}

impl Drop for BonjourMdnsService {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
    }
}

#[derive(Default, FromRaw)]
struct BonjourServiceContext {
    registered_callback: Option<Box<ServiceRegisteredCallback>>,
    user_context: Option<Arc<dyn Any>>,
}

impl BonjourServiceContext {
    fn invoke_callback(&self, result: Result<ServiceRegistration>) {
        if let Some(f) = &self.registered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke callback but none was set");
        }
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

    let result = ServiceRegistration::builder()
        .name(c_str::copy_raw(name))
        .kind(c_str::copy_raw(regtype))
        .domain(domain)
        .build()
        .expect("could not build ServiceRegistration");

    context.invoke_callback(Ok(result));

    Ok(())
}
