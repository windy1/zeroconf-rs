use super::client::{self, ManagedAvahiClient, ManagedAvahiClientParams};
use super::constants;
use super::entry_group::{AddServiceParams, ManagedAvahiEntryGroup, ManagedAvahiEntryGroupParams};
use super::poll::ManagedAvahiSimplePoll;
use crate::builder::BuilderDelegate;
use crate::ffi::{cstr, AsRaw, FromRaw};
use crate::{ServiceRegisteredCallback, ServiceRegistration};
use avahi_sys::{
    AvahiClient, AvahiClientFlags, AvahiClientState, AvahiEntryGroup, AvahiEntryGroupState,
};
use libc::c_void;
use std::any::Any;
use std::ffi::CString;
use std::fmt::{self, Formatter};
use std::ptr;
use std::sync::Arc;

/// Interface for interacting with Avahi's mDNS service registration capabilities.
#[derive(Debug)]
pub struct AvahiMdnsService {
    client: Option<ManagedAvahiClient>,
    poll: Option<ManagedAvahiSimplePoll>,
    context: *mut AvahiServiceContext,
}

impl AvahiMdnsService {
    pub fn new(kind: &str, port: u16) -> Self {
        Self {
            client: None,
            poll: None,
            context: Box::into_raw(Box::new(AvahiServiceContext::new(kind, port))),
        }
    }

    pub fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>) {
        unsafe { (*self.context).registered_callback = Some(registered_callback) };
    }

    pub fn set_context(&mut self, context: Box<dyn Any>) {
        unsafe { (*self.context).user_context = Some(Arc::from(context)) };
    }

    pub fn start(&mut self) -> Result<(), String> {
        debug!("Registering service: {:?}", self);

        self.poll = Some(ManagedAvahiSimplePoll::new()?);

        self.client = Some(ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(self.poll.as_ref().unwrap())
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(self.context as *mut c_void)
                .build()?,
        )?);

        self.poll.as_ref().unwrap().start_loop()
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
    registered_callback: Option<Box<ServiceRegisteredCallback>>,
    user_context: Option<Arc<dyn Any>>,
}

impl AvahiServiceContext {
    fn new(kind: &str, port: u16) -> Self {
        Self {
            name: None,
            kind: CString::new(kind).unwrap(),
            port,
            group: None,
            registered_callback: None,
            user_context: None,
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
        avahi_sys::AvahiClientState_AVAHI_CLIENT_S_RUNNING => create_service(client, context),
        avahi_sys::AvahiClientState_AVAHI_CLIENT_FAILURE => panic!("client failure"),
        avahi_sys::AvahiClientState_AVAHI_CLIENT_S_REGISTERING => {
            if let Some(g) = &mut context.group {
                debug!("Group reset");
                g.reset();
            }
        }
        _ => {}
    };
}

unsafe fn create_service(client: *mut AvahiClient, context: &mut AvahiServiceContext) {
    context.name = Some(CString::new(client::get_host_name(client).unwrap().to_string()).unwrap());

    if context.group.is_none() {
        debug!("Creating group");

        context.group = Some(
            ManagedAvahiEntryGroup::new(
                ManagedAvahiEntryGroupParams::builder()
                    .client(client)
                    .callback(Some(entry_group_callback))
                    .userdata(context.as_raw())
                    .build()
                    .unwrap(),
            )
            .unwrap(),
        );
    }

    let group = context.group.as_mut().unwrap();

    if group.is_empty() {
        debug!("Adding service");

        group
            .add_service(
                AddServiceParams::builder()
                    .interface(constants::AVAHI_IF_UNSPEC)
                    .protocol(constants::AVAHI_PROTO_UNSPEC)
                    .flags(0)
                    .name(context.name.as_ref().unwrap().as_ptr())
                    .kind(context.kind.as_ptr())
                    .domain(ptr::null_mut())
                    .host(ptr::null_mut())
                    .port(context.port)
                    .build()
                    .unwrap(),
            )
            .unwrap();
    }
}

unsafe extern "C" fn entry_group_callback(
    _group: *mut AvahiEntryGroup,
    state: AvahiEntryGroupState,
    userdata: *mut c_void,
) {
    match state {
        avahi_sys::AvahiEntryGroupState_AVAHI_ENTRY_GROUP_ESTABLISHED => {
            debug!("Group established");

            let context = AvahiServiceContext::from_raw(userdata);

            let result = ServiceRegistration::builder()
                .name(cstr::copy_raw(context.name.as_ref().unwrap().as_ptr()))
                .kind(cstr::copy_raw(context.kind.as_ptr()))
                .domain("local".to_string())
                .build()
                .expect("could not build ServiceRegistration");

            if let Some(f) = &context.registered_callback {
                f(result, context.user_context.clone());
            } else {
                warn!("Service registered but no callback was set: {:?}", result);
            }
        }
        _ => {}
    };
}
