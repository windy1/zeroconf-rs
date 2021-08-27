//! Avahi implementation for cross-platform service.

use super::avahi_util;
use super::client::{self, ManagedAvahiClient, ManagedAvahiClientParams};
use super::entry_group::{AddServiceParams, ManagedAvahiEntryGroup, ManagedAvahiEntryGroupParams};
use super::poll::ManagedAvahiSimplePoll;
use crate::ffi::{c_str, AsRaw, FromRaw, UnwrapOrNull};
use crate::prelude::*;
use crate::service::ServiceRegisterFuture;
use crate::{
    EventLoop, NetworkInterface, Result, ServiceRegisteredCallback, ServiceRegistration,
    ServiceType, TxtRecord,
};
use avahi_sys::{
    AvahiClient, AvahiClientFlags, AvahiClientState, AvahiEntryGroup, AvahiEntryGroupState,
    AvahiIfIndex,
};
use libc::c_void;
use std::any::Any;
use std::ffi::CString;
use std::fmt::{self, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;

#[derive(Debug)]
pub struct AvahiMdnsService {
    client: Option<ManagedAvahiClient>,
    event_loop: Option<EventLoop>,
    timeout: Duration,
    context: *mut AvahiServiceContext,
}

impl TMdnsService for AvahiMdnsService {
    fn new(service_type: ServiceType, port: u16) -> Self {
        Self {
            client: None,
            event_loop: None,
            timeout: Duration::from_secs(0),
            context: Box::into_raw(Box::new(AvahiServiceContext::new(
                &service_type.to_string(),
                port,
            ))),
        }
    }

    /// Sets the name to register this service under. If no name is set, the client's host name
    /// will be used instead.
    ///
    /// See: [`AvahiClient::host_name()`]
    ///
    /// [`AvahiClient::host_name()`]: client/struct.ManagedAvahiClient.html#method.host_name
    fn set_name(&mut self, name: &str) {
        unsafe { (*self.context).name = Some(c_string!(name)) };
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        unsafe { (*self.context).interface_index = avahi_util::interface_index(interface) };
    }

    fn set_domain(&mut self, domain: &str) {
        unsafe { (*self.context).domain = Some(c_string!(domain)) };
    }

    fn set_host(&mut self, host: &str) {
        unsafe { (*self.context).host = Some(c_string!(host)) };
    }

    fn set_txt_record(&mut self, txt_record: TxtRecord) {
        unsafe { (*self.context).txt_record = Some(txt_record) };
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

        let poll = ManagedAvahiSimplePoll::new()?;

        self.client = Some(ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(&poll)
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(self.context as *mut c_void)
                .build()?,
        )?);

        self.event_loop = Some(EventLoop::new(poll));

        Ok(self.event_loop.as_ref().unwrap())
    }

    fn register_async(&mut self) -> ServiceRegisterFuture {
        Box::pin(AvahiServiceRegisterFuture::new(self))
    }
}

impl Drop for AvahiMdnsService {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
    }
}

#[derive(new)]
struct AvahiServiceRegisterFuture<'a> {
    service: &'a mut AvahiMdnsService,
}

impl<'a> Future for AvahiServiceRegisterFuture<'a> {
    type Output = Result<ServiceRegistration>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = ctx.waker();
        let service = &mut self.service;
        if let Some(result) = unsafe { (*service.context).registration_result.take() } {
            Poll::Ready(result)
        } else if let Some(event_loop) = &service.event_loop {
            if let Err(error) = event_loop.poll(service.timeout) {
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

#[derive(FromRaw, AsRaw)]
struct AvahiServiceContext {
    name: Option<CString>,
    kind: CString,
    port: u16,
    group: Option<ManagedAvahiEntryGroup>,
    txt_record: Option<TxtRecord>,
    interface_index: AvahiIfIndex,
    domain: Option<CString>,
    host: Option<CString>,
    registration_result: Option<Result<ServiceRegistration>>,
    registered_callback: Option<Box<ServiceRegisteredCallback>>,
    user_context: Option<Arc<dyn Any>>,
}

impl AvahiServiceContext {
    fn new(kind: &str, port: u16) -> Self {
        Self {
            name: None,
            kind: c_string!(kind),
            port,
            group: None,
            txt_record: None,
            interface_index: avahi_sys::AVAHI_IF_UNSPEC,
            domain: None,
            host: None,
            registration_result: None,
            registered_callback: None,
            user_context: None,
        }
    }

    fn invoke_callback(&mut self, result: Result<ServiceRegistration>) {
        self.registration_result = Some(result.clone());
        if let Some(f) = &self.registered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke service callback but none was set");
        }
    }
}

impl fmt::Debug for AvahiServiceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AvahiServiceContext")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("port", &self.port)
            .field("group", &self.group)
            .field("txt_record", &self.txt_record)
            .field("interface_index", &self.interface_index)
            .field("domain", &self.domain)
            .field("host", &self.host)
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

unsafe extern "C" fn client_callback(
    client: *mut AvahiClient,
    state: AvahiClientState,
    userdata: *mut c_void,
) {
    let context = AvahiServiceContext::from_raw(userdata);

    match state {
        avahi_sys::AvahiClientState_AVAHI_CLIENT_S_RUNNING => {
            if let Err(e) = create_service(client, context) {
                context.invoke_callback(Err(e));
            }
        }
        avahi_sys::AvahiClientState_AVAHI_CLIENT_FAILURE => {
            context.invoke_callback(Err("client failure".into()))
        }
        avahi_sys::AvahiClientState_AVAHI_CLIENT_S_REGISTERING => {
            if let Some(g) = &mut context.group {
                debug!("Group reset");
                g.reset();
            }
        }
        _ => {}
    };
}

unsafe fn create_service(
    client: *mut AvahiClient,
    context: &mut AvahiServiceContext,
) -> Result<()> {
    if context.name.is_none() {
        context.name = Some(c_string!(client::get_host_name(client)?.to_string()));
    }

    if context.group.is_none() {
        debug!("Creating group");

        context.group = Some(ManagedAvahiEntryGroup::new(
            ManagedAvahiEntryGroupParams::builder()
                .client(client)
                .callback(Some(entry_group_callback))
                .userdata(context.as_raw())
                .build()?,
        )?);
    }

    let group = context.group.as_mut().unwrap();

    if group.is_empty() {
        debug!("Adding service");

        group.add_service(
            AddServiceParams::builder()
                .interface(context.interface_index)
                .protocol(avahi_sys::AVAHI_PROTO_UNSPEC)
                .flags(0)
                .name(context.name.as_ref().unwrap().as_ptr())
                .kind(context.kind.as_ptr())
                .domain(context.domain.as_ref().map(|d| d.as_ptr()).unwrap_or_null())
                .host(context.host.as_ref().map(|h| h.as_ptr()).unwrap_or_null())
                .port(context.port)
                .txt(context.txt_record.as_ref().map(|t| t.inner()))
                .build()?,
        )
    } else {
        Ok(())
    }
}

unsafe extern "C" fn entry_group_callback(
    _group: *mut AvahiEntryGroup,
    state: AvahiEntryGroupState,
    userdata: *mut c_void,
) {
    if let avahi_sys::AvahiEntryGroupState_AVAHI_ENTRY_GROUP_ESTABLISHED = state {
        let context = AvahiServiceContext::from_raw(userdata);
        if let Err(e) = handle_group_established(context) {
            context.invoke_callback(Err(e));
        }
    }
}

unsafe fn handle_group_established(context: &mut AvahiServiceContext) -> Result<()> {
    debug!("Group established");

    let result = ServiceRegistration::builder()
        .name(c_str::copy_raw(context.name.as_ref().unwrap().as_ptr()))
        .service_type(ServiceType::from_str(&c_str::copy_raw(
            context.kind.as_ptr(),
        ))?)
        .domain("local".to_string())
        .build()?;

    context.invoke_callback(Ok(result));

    Ok(())
}
