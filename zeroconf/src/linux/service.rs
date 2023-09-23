//! Avahi implementation for cross-platform service.

use super::avahi_util;
use super::client::{self, ManagedAvahiClient, ManagedAvahiClientParams};
use super::entry_group::{
    AddServiceParams, AddServiceSubtypeParams, ManagedAvahiEntryGroup, ManagedAvahiEntryGroupParams,
};
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
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AvahiMdnsService {
    client: Option<Rc<ManagedAvahiClient>>,
    poll: Option<Rc<ManagedAvahiSimplePoll>>,
    context: Box<AvahiServiceContext>,
}

impl TMdnsService for AvahiMdnsService {
    fn new(service_type: ServiceType, port: u16) -> Self {
        let kind = avahi_util::format_service_type(&service_type);

        let sub_types = service_type
            .sub_types()
            .iter()
            .map(|sub_type| c_string!(avahi_util::format_sub_type(sub_type, &kind)))
            .collect::<Vec<_>>();

        Self {
            client: None,
            poll: None,
            context: Box::new(AvahiServiceContext::new(c_string!(kind), port, sub_types)),
        }
    }

    /// Sets the name to register this service under. If no name is set, the client's host name
    /// will be used instead.
    ///
    /// See: [`AvahiClient::host_name()`]
    ///
    /// [`AvahiClient::host_name()`]: client/struct.ManagedAvahiClient.html#method.host_name
    fn set_name(&mut self, name: &str) {
        self.context.name = Some(c_string!(name))
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.context.interface_index = avahi_util::interface_index(interface)
    }

    fn set_domain(&mut self, domain: &str) {
        self.context.domain = Some(c_string!(domain))
    }

    fn set_host(&mut self, host: &str) {
        self.context.host = Some(c_string!(host))
    }

    fn set_txt_record(&mut self, txt_record: TxtRecord) {
        self.context.txt_record = Some(txt_record)
    }

    fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>) {
        self.context.registered_callback = Some(registered_callback)
    }

    fn set_context(&mut self, context: Box<dyn Any>) {
        self.context.user_context = Some(Arc::from(context))
    }

    fn register(&mut self) -> Result<EventLoop> {
        debug!("Registering service: {:?}", self);

        self.poll = Some(Rc::new(ManagedAvahiSimplePoll::new()?));

        self.client = Some(Rc::new(ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(Rc::clone(self.poll.as_ref().unwrap()))
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(self.context.as_raw())
                .build()?,
        )?));

        self.context.client = self.client.clone();

        unsafe { create_service(&mut self.context) }?;

        Ok(EventLoop::new(self.poll.as_ref().unwrap().clone()))
    }
}

#[derive(FromRaw, AsRaw)]
struct AvahiServiceContext {
    client: Option<Rc<ManagedAvahiClient>>,
    name: Option<CString>,
    kind: CString,
    sub_types: Vec<CString>,
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
    fn new(kind: CString, port: u16, sub_types: Vec<CString>) -> Self {
        Self {
            client: None,
            name: None,
            kind,
            port,
            sub_types,
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
            panic!("attempted to invoke service callback but none was set");
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

unsafe fn create_service(context: &mut AvahiServiceContext) -> Result<()> {
    if context.name.is_none() {
        let host_name = client::get_host_name(context.client.as_ref().unwrap().inner)?;
        context.name = Some(c_string!(host_name.to_string()));
    }

    if context.group.is_none() {
        debug!("Creating group");

        context.group = Some(ManagedAvahiEntryGroup::new(
            ManagedAvahiEntryGroupParams::builder()
                .client(Rc::clone(context.client.as_ref().unwrap()))
                .callback(Some(entry_group_callback))
                .userdata(context.as_raw())
                .build()?,
        )?);
    }

    let group = context.group.as_mut().unwrap();

    if !group.is_empty() {
        return Ok(());
    }

    debug!("Adding service: {}", context.kind.to_string_lossy());

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
    )?;

    for sub_type in &context.sub_types {
        debug!("Adding service subtype: {}", sub_type.to_string_lossy());

        group.add_service_subtype(
            AddServiceSubtypeParams::builder()
                .interface(context.interface_index)
                .protocol(avahi_sys::AVAHI_PROTO_UNSPEC)
                .flags(0)
                .name(context.name.as_ref().unwrap().as_ptr())
                .kind(context.kind.as_ptr())
                .domain(context.domain.as_ref().map(|d| d.as_ptr()).unwrap_or_null())
                .subtype(sub_type.as_ptr())
                .build()?,
        )?;
    }

    group.commit()
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

unsafe extern "C" fn client_callback(
    _client: *mut AvahiClient,
    state: AvahiClientState,
    _userdata: *mut c_void,
) {
    // TODO: handle this better
    if let avahi_sys::AvahiClientState_AVAHI_CLIENT_FAILURE = state {
        panic!("client failure");
    }
}
