//! Avahi implementation for cross-platform service.

use super::avahi_util;
use super::client::{self, ManagedAvahiClient, ManagedAvahiClientParams};
use super::entry_group::{AddServiceParams, ManagedAvahiEntryGroup, ManagedAvahiEntryGroupParams};
use super::poll::ManagedAvahiSimplePoll;
use crate::ffi::{c_str, AsRaw, FromRaw, UnwrapOrNull};
use crate::prelude::*;
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
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AvahiMdnsService {
    client: Option<ManagedAvahiClient>,
    poll: Option<Arc<ManagedAvahiSimplePoll>>,
    context: *mut AvahiServiceContext,
}

impl TMdnsService for AvahiMdnsService {
    fn new(service_type: ServiceType, port: u16) -> Self {
        Self {
            client: None,
            poll: None,
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

    fn register(&mut self) -> Result<EventLoop> {
        debug!("Registering service: {:?}", self);

        self.poll = Some(Arc::new(ManagedAvahiSimplePoll::new()?));

        self.client = Some(ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(self.poll.as_ref().unwrap())
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(self.context as *mut c_void)
                .build()?,
        )?);

        Ok(EventLoop::new(self.poll.as_ref().unwrap().clone()))
    }
}

impl Drop for AvahiMdnsService {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
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
            registered_callback: None,
            user_context: None,
        }
    }

    fn invoke_callback(&self, result: Result<ServiceRegistration>) {
        if let Some(f) = &self.registered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke callback but none was set");
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

unsafe fn handle_group_established(context: &AvahiServiceContext) -> Result<()> {
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
