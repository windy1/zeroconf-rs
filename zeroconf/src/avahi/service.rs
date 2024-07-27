//! Avahi implementation for cross-platform service.

use super::avahi_util;
use super::client::{ManagedAvahiClient, ManagedAvahiClientParams};
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
use std::ffi::{CStr, CString};
use std::fmt::{self, Formatter};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AvahiMdnsService {
    // note: this declaration order is important, it ensures that each
    // component is dropped in the correct order
    context: Box<AvahiServiceContext>,
    client: Option<Arc<ManagedAvahiClient>>,
    poll: Option<Arc<ManagedAvahiSimplePoll>>,
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
        self.context.name = c_string!(name).into()
    }

    fn name(&self) -> Option<&str> {
        self.context.name.as_ref().map(c_str::to_str)
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.context.interface_index = avahi_util::interface_index(interface)
    }

    fn network_interface(&self) -> NetworkInterface {
        avahi_util::interface_from_index(self.context.interface_index)
    }

    fn set_domain(&mut self, domain: &str) {
        self.context.domain = c_string!(domain).into()
    }

    fn domain(&self) -> Option<&str> {
        self.context.domain.as_ref().map(c_str::to_str)
    }

    fn set_host(&mut self, host: &str) {
        self.context.host = c_string!(host).into()
    }

    fn host(&self) -> Option<&str> {
        self.context.host.as_ref().map(c_str::to_str)
    }

    fn set_txt_record(&mut self, txt_record: TxtRecord) {
        self.context.txt_record = txt_record.into()
    }

    fn txt_record(&self) -> Option<&TxtRecord> {
        self.context.txt_record.as_ref()
    }

    fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>) {
        self.context.registered_callback = registered_callback.into()
    }

    fn set_context(&mut self, context: Box<dyn Any>) {
        self.context.user_context = Some(Arc::from(context))
    }

    fn context(&self) -> Option<&dyn Any> {
        self.context.user_context.as_ref().map(|c| c.as_ref())
    }

    fn register(&mut self) -> Result<EventLoop> {
        debug!("Registering service: {:?}", self);

        self.poll = Some(Arc::new(ManagedAvahiSimplePoll::new()?));

        self.client = Some(Arc::new(ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(
                    self.poll
                        .as_ref()
                        .ok_or("could not get poll as ref")?
                        .clone(),
                )
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(self.context.as_raw())
                .build()?,
        )?));

        self.context.client.clone_from(&self.client);

        unsafe {
            if let Err(e) = create_service(&mut self.context) {
                self.context.invoke_callback(Err(e))
            }
        }

        Ok(EventLoop::new(
            self.poll
                .as_ref()
                .ok_or("could not get poll as ref")?
                .clone(),
        ))
    }
}

#[derive(FromRaw, AsRaw)]
struct AvahiServiceContext {
    client: Option<Arc<ManagedAvahiClient>>,
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
        avahi_sys::AvahiServerState_AVAHI_SERVER_INVALID
        | avahi_sys::AvahiServerState_AVAHI_SERVER_COLLISION
        | avahi_sys::AvahiServerState_AVAHI_SERVER_FAILURE => {
            context.invoke_callback(Err(avahi_util::get_last_error(client).into()))
        }
        _ => {}
    }
}

unsafe fn create_service(context: &mut AvahiServiceContext) -> Result<()> {
    if context.name.is_none() {
        let host_name = context
            .client
            .as_ref()
            .ok_or("expected initialized client")?
            .host_name()?;

        context.name = Some(c_string!(host_name.to_string()));
    }

    if context.group.is_none() {
        debug!("Creating group");

        context.group = Some(ManagedAvahiEntryGroup::new(
            ManagedAvahiEntryGroupParams::builder()
                .client(Arc::clone(
                    context
                        .client
                        .as_ref()
                        .ok_or("could not get client as ref")?,
                ))
                .callback(Some(entry_group_callback))
                .userdata(context.as_raw())
                .build()?,
        )?);
    }

    let group = context
        .group
        .as_mut()
        .ok_or("could not borrow group as mut")?;

    if !group.is_empty() {
        return Ok(());
    }

    let name = context
        .name
        .as_ref()
        .ok_or("could not get name as ref")?
        .clone();

    add_services(context, &name)
}

fn add_services(context: &mut AvahiServiceContext, name: &CStr) -> Result<()> {
    debug!("Adding service: {}", context.kind.to_string_lossy());

    let group = context
        .group
        .as_mut()
        .ok_or("could not borrow group as mut")?;

    group.add_service(
        AddServiceParams::builder()
            .interface(context.interface_index)
            .protocol(avahi_sys::AVAHI_PROTO_UNSPEC)
            .flags(0)
            .name(name.as_ptr())
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
                .name(name.as_ptr())
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
    let context = AvahiServiceContext::from_raw(userdata);

    let client = context
        .client
        .as_ref()
        .expect("expected initialized client");

    match state {
        avahi_sys::AvahiEntryGroupState_AVAHI_ENTRY_GROUP_ESTABLISHED => {
            context.invoke_callback(handle_group_established(context))
        }
        avahi_sys::AvahiEntryGroupState_AVAHI_ENTRY_GROUP_FAILURE => {
            context.invoke_callback(Err(avahi_util::get_last_error(client.inner).into()))
        }
        avahi_sys::AvahiEntryGroupState_AVAHI_ENTRY_GROUP_COLLISION => {
            let name = context
                .name
                .as_ref()
                .expect("expected initialized name")
                .clone();

            let new_name = avahi_util::alternative_service_name(name.as_c_str());
            let result = add_services(context, new_name);

            context.name = Some(new_name.into());

            if let Err(e) = result {
                context.invoke_callback(Err(e))
            }
        }
        _ => {}
    }
}

unsafe fn handle_group_established(context: &AvahiServiceContext) -> Result<ServiceRegistration> {
    debug!("Group established");

    let name = c_str::copy_raw(
        context
            .name
            .as_ref()
            .ok_or("could not get name as ref")?
            .as_ptr(),
    );

    Ok(ServiceRegistration::builder()
        .name(name)
        .service_type(ServiceType::from_str(&c_str::copy_raw(
            context.kind.as_ptr(),
        ))?)
        .domain("local".to_string())
        .build()?)
}
