//! Avahi implementation for cross-platform browser

use super::avahi_util;
use super::client::{ManagedAvahiClient, ManagedAvahiClientParams};
use super::poll::ManagedAvahiSimplePoll;
use super::raw_browser::{ManagedAvahiServiceBrowser, ManagedAvahiServiceBrowserParams};
use super::{
    resolver::{
        ManagedAvahiServiceResolver, ManagedAvahiServiceResolverParams, ServiceResolverSet,
    },
    string_list::ManagedAvahiStringList,
};
use crate::ffi::{c_str, AsRaw, FromRaw};
use crate::prelude::*;
use crate::Result;
use crate::{
    EventLoop, NetworkInterface, ServiceDiscoveredCallback, ServiceDiscovery, ServiceType,
    TxtRecord,
};
use avahi_sys::{
    AvahiAddress, AvahiBrowserEvent, AvahiClient, AvahiClientFlags, AvahiClientState, AvahiIfIndex,
    AvahiLookupResultFlags, AvahiProtocol, AvahiResolverEvent, AvahiServiceBrowser,
    AvahiServiceResolver, AvahiStringList,
};
use libc::{c_char, c_void};
use std::any::Any;
use std::ffi::CString;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt, ptr};

#[derive(Debug)]
pub struct AvahiMdnsBrowser {
    client: Option<Arc<ManagedAvahiClient>>,
    poll: Option<Arc<ManagedAvahiSimplePoll>>,
    browser: Option<ManagedAvahiServiceBrowser>,
    kind: CString,
    interface_index: AvahiIfIndex,
    context: *mut AvahiBrowserContext,
}

impl TMdnsBrowser for AvahiMdnsBrowser {
    fn new(service_type: ServiceType) -> Self {
        Self {
            client: None,
            poll: None,
            browser: None,
            kind: c_string!(service_type.to_string()),
            context: Box::into_raw(Box::default()),
            interface_index: avahi_sys::AVAHI_IF_UNSPEC,
        }
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = avahi_util::interface_index(interface);
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

    fn browse_services(&mut self) -> Result<EventLoop> {
        debug!("Browsing services: {:?}", self);

        self.poll = Some(Arc::new(ManagedAvahiSimplePoll::new()?));

        self.client = Some(Arc::new(ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(self.poll.as_ref().unwrap())
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(ptr::null_mut())
                .build()?,
        )?));

        unsafe {
            (*self.context).client = self.client.clone();

            self.browser = Some(ManagedAvahiServiceBrowser::new(
                ManagedAvahiServiceBrowserParams::builder()
                    .client(&(*self.context).client.as_ref().unwrap())
                    .interface(self.interface_index)
                    .protocol(avahi_sys::AVAHI_PROTO_UNSPEC)
                    .kind(self.kind.as_ptr())
                    .domain(ptr::null_mut())
                    .flags(0)
                    .callback(Some(browse_callback))
                    .userdata(self.context as *mut c_void)
                    .build()?,
            )?);
        }

        Ok(EventLoop::new(self.poll.as_ref().unwrap().clone()))
    }
}

impl Drop for AvahiMdnsBrowser {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.context) };
        // browser must be freed first
        self.browser = None;
    }
}

#[derive(FromRaw, AsRaw)]
struct AvahiBrowserContext {
    client: Option<Arc<ManagedAvahiClient>>,
    resolvers: ServiceResolverSet,
    service_discovered_callback: Option<Box<ServiceDiscoveredCallback>>,
    user_context: Option<Arc<dyn Any>>,
}

impl AvahiBrowserContext {
    fn invoke_callback(&self, result: Result<ServiceDiscovery>) {
        if let Some(f) = &self.service_discovered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke callback but none was set");
        }
    }
}

impl Default for AvahiBrowserContext {
    fn default() -> Self {
        AvahiBrowserContext {
            client: None,
            resolvers: ServiceResolverSet::default(),
            service_discovered_callback: None,
            user_context: None,
        }
    }
}

impl fmt::Debug for AvahiBrowserContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AvahiBrowserContext")
            .field("client", &self.client)
            .field("resolvers", &self.resolvers)
            .finish()
    }
}

unsafe extern "C" fn browse_callback(
    _browser: *mut AvahiServiceBrowser,
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    event: AvahiBrowserEvent,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    _flags: AvahiLookupResultFlags,
    userdata: *mut c_void,
) {
    let context = AvahiBrowserContext::from_raw(userdata);

    match event {
        avahi_sys::AvahiBrowserEvent_AVAHI_BROWSER_NEW => {
            if let Err(e) = handle_browser_new(context, interface, protocol, name, kind, domain) {
                context.invoke_callback(Err(e));
            }
        }
        avahi_sys::AvahiBrowserEvent_AVAHI_BROWSER_FAILURE => {
            context.invoke_callback(Err("browser failure".into()))
        }
        _ => {}
    };
}

fn handle_browser_new(
    context: &mut AvahiBrowserContext,
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
) -> Result<()> {
    let raw_context = context.as_raw();
    context.resolvers.insert(ManagedAvahiServiceResolver::new(
        ManagedAvahiServiceResolverParams::builder()
            .client(context.client.as_ref().unwrap())
            .interface(interface)
            .protocol(protocol)
            .name(name)
            .kind(kind)
            .domain(domain)
            .aprotocol(avahi_sys::AVAHI_PROTO_UNSPEC)
            .flags(0)
            .callback(Some(resolve_callback))
            .userdata(raw_context)
            .build()?,
    )?);
    Ok(())
}

unsafe extern "C" fn resolve_callback(
    resolver: *mut AvahiServiceResolver,
    _interface: AvahiIfIndex,
    _protocol: AvahiProtocol,
    event: AvahiResolverEvent,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    host_name: *const c_char,
    addr: *const AvahiAddress,
    port: u16,
    txt: *mut AvahiStringList,
    _flags: AvahiLookupResultFlags,
    userdata: *mut c_void,
) {
    let name = c_str::raw_to_str(name);
    let kind = c_str::raw_to_str(kind);
    let domain = c_str::raw_to_str(domain);

    let context = AvahiBrowserContext::from_raw(userdata);

    match event {
        avahi_sys::AvahiResolverEvent_AVAHI_RESOLVER_FAILURE => {
            context.invoke_callback(Err(format!(
                "failed to resolve service `{}` of type `{}` in domain `{}`",
                name, kind, domain
            )
            .into()));
        }
        avahi_sys::AvahiResolverEvent_AVAHI_RESOLVER_FOUND => {
            let result = handle_resolver_found(
                context,
                c_str::raw_to_str(host_name),
                addr,
                name,
                kind,
                domain,
                port,
                txt,
            );

            if let Err(e) = result {
                context.invoke_callback(Err(e));
            }
        }
        _ => {}
    };

    context.resolvers.remove_raw(resolver);
}

#[allow(clippy::too_many_arguments)]
unsafe fn handle_resolver_found(
    context: &AvahiBrowserContext,
    host_name: &str,
    addr: *const AvahiAddress,
    name: &str,
    kind: &str,
    domain: &str,
    port: u16,
    txt: *mut AvahiStringList,
) -> Result<()> {
    let address = avahi_util::avahi_address_to_string(addr);

    let txt = if txt.is_null() {
        None
    } else {
        Some(TxtRecord::from(ManagedAvahiStringList::clone_raw(txt)))
    };

    let result = ServiceDiscovery::builder()
        .name(name.to_string())
        .service_type(ServiceType::from_str(kind)?)
        .domain(domain.to_string())
        .host_name(host_name.to_string())
        .address(address)
        .port(port)
        .txt(txt)
        .build()
        .unwrap();

    debug!("Service resolved: {:?}", result);

    context.invoke_callback(Ok(result));

    Ok(())
}

extern "C" fn client_callback(
    _client: *mut AvahiClient,
    state: AvahiClientState,
    _userdata: *mut c_void,
) {
    // TODO: handle this better
    if let avahi_sys::AvahiClientState_AVAHI_CLIENT_FAILURE = state {
        panic!("client failure");
    }
}
