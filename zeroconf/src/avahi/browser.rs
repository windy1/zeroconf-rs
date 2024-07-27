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
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt, ptr};

#[derive(Debug)]
pub struct AvahiMdnsBrowser {
    context: Box<AvahiBrowserContext>,
    client: Option<Arc<ManagedAvahiClient>>,
    poll: Option<Arc<ManagedAvahiSimplePoll>>,
}

impl TMdnsBrowser for AvahiMdnsBrowser {
    fn new(service_type: ServiceType) -> Self {
        Self {
            client: None,
            poll: None,
            context: Box::new(AvahiBrowserContext::new(
                c_string!(avahi_util::format_browser_type(&service_type)),
                avahi_sys::AVAHI_IF_UNSPEC,
            )),
        }
    }

    fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.context.interface_index = avahi_util::interface_index(interface);
    }

    fn network_interface(&self) -> NetworkInterface {
        avahi_util::interface_from_index(self.context.interface_index)
    }

    fn set_service_discovered_callback(
        &mut self,
        service_discovered_callback: Box<ServiceDiscoveredCallback>,
    ) {
        self.context.service_discovered_callback = Some(service_discovered_callback);
    }

    fn set_context(&mut self, context: Box<dyn Any>) {
        self.context.user_context = Some(Arc::from(context));
    }

    fn context(&self) -> Option<&dyn Any> {
        self.context.user_context.as_ref().map(|c| c.as_ref())
    }

    fn browse_services(&mut self) -> Result<EventLoop> {
        debug!("Browsing services: {:?}", self);

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
            if let Err(e) = create_browser(&mut self.context) {
                self.context.invoke_callback(Err(e));
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
struct AvahiBrowserContext {
    client: Option<Arc<ManagedAvahiClient>>,
    resolvers: ServiceResolverSet,
    service_discovered_callback: Option<Box<ServiceDiscoveredCallback>>,
    user_context: Option<Arc<dyn Any>>,
    interface_index: AvahiIfIndex,
    kind: CString,
    browser: Option<ManagedAvahiServiceBrowser>,
}

impl AvahiBrowserContext {
    fn new(kind: CString, interface_index: AvahiIfIndex) -> Self {
        Self {
            client: None,
            resolvers: ServiceResolverSet::default(),
            service_discovered_callback: None,
            user_context: None,
            interface_index,
            kind,
            browser: None,
        }
    }

    fn invoke_callback(&self, result: Result<ServiceDiscovery>) {
        if let Some(f) = &self.service_discovered_callback {
            f(result, self.user_context.clone());
        } else {
            warn!("attempted to invoke browser callback but none was set");
        }
    }
}

impl fmt::Debug for AvahiBrowserContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AvahiBrowserContext")
            .field("resolvers", &self.resolvers)
            .finish()
    }
}

unsafe extern "C" fn client_callback(
    client: *mut AvahiClient,
    state: AvahiClientState,
    userdata: *mut c_void,
) {
    let context = AvahiBrowserContext::from_raw(userdata);

    if state == avahi_sys::AvahiClientState_AVAHI_CLIENT_FAILURE {
        context.invoke_callback(Err(avahi_util::get_last_error(client).into()));
    }
}

unsafe fn create_browser(context: &mut AvahiBrowserContext) -> Result<()> {
    context.browser = Some(ManagedAvahiServiceBrowser::new(
        ManagedAvahiServiceBrowserParams::builder()
            .interface(context.interface_index)
            .protocol(avahi_sys::AVAHI_PROTO_UNSPEC)
            .kind(context.kind.as_ptr())
            .domain(ptr::null_mut())
            .flags(0)
            .callback(Some(browse_callback))
            .userdata(context.as_raw())
            .client(Arc::clone(
                context
                    .client
                    .as_ref()
                    .ok_or("could not get client as ref")?,
            ))
            .build()?,
    )?);

    Ok(())
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

unsafe fn handle_browser_new(
    context: &mut AvahiBrowserContext,
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
) -> Result<()> {
    let raw_context = context.as_raw();

    let client = context
        .client
        .as_ref()
        .ok_or("expected initialized client")?;

    context.resolvers.insert(ManagedAvahiServiceResolver::new(
        ManagedAvahiServiceResolverParams::builder()
            .client(client.clone())
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
        .build()?;

    debug!("Service resolved: {:?}", result);

    context.invoke_callback(Ok(result));

    Ok(())
}
