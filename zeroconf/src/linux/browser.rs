use super::avahi_util;
use super::client::{ManagedAvahiClient, ManagedAvahiClientParams};
use super::constants;
use super::poll::ManagedAvahiSimplePoll;
use super::raw_browser::{ManagedAvahiServiceBrowser, ManagedAvahiServiceBrowserParams};
use super::resolver::{
    ManagedAvahiServiceResolver, ManagedAvahiServiceResolverParams, ServiceResolverSet,
};
use crate::builder::BuilderDelegate;
use crate::ffi::{cstr, AsRaw, FromRaw};
use crate::Result;
use crate::{NetworkInterface, ServiceDiscoveredCallback, ServiceDiscovery};
use avahi_sys::{
    AvahiAddress, AvahiBrowserEvent, AvahiClient, AvahiClientFlags, AvahiClientState, AvahiIfIndex,
    AvahiLookupResultFlags, AvahiProtocol, AvahiResolverEvent, AvahiServiceBrowser,
    AvahiServiceResolver, AvahiStringList,
};
use libc::{c_char, c_void};
use std::any::Any;
use std::ffi::CString;
use std::sync::Arc;
use std::{fmt, ptr};

/// Interface for interacting with Avahi's mDNS service browsing capabilities.
#[derive(Debug)]
pub struct AvahiMdnsBrowser {
    poll: Option<ManagedAvahiSimplePoll>,
    browser: Option<ManagedAvahiServiceBrowser>,
    kind: CString,
    interface_index: AvahiIfIndex,
    context: *mut AvahiBrowserContext,
}

impl AvahiMdnsBrowser {
    /// Creates a new `AvahiMdnsBrowser` that browses for the specified `kind` (e.g. `_http._tcp`)
    pub fn new(kind: &str) -> Self {
        Self {
            poll: None,
            browser: None,
            kind: c_string!(kind.to_string()),
            context: Box::into_raw(Box::default()),
            interface_index: constants::AVAHI_IF_UNSPEC,
        }
    }

    /// Sets the network interface on which to browse for services on.
    ///
    /// Most applications will want to use the default value `NetworkInterface::Unspec` to browse
    /// on all available interfaces.
    pub fn set_network_interface(&mut self, interface: NetworkInterface) {
        self.interface_index = avahi_util::interface_index(interface);
    }

    /// Sets the [`ServiceDiscoveredCallback`] that is invoked when the browser has discovered and
    /// resolved a service.
    ///
    /// [`ServiceDiscoveredCallback`]: ../type.ServiceDiscoveredCallback.html
    pub fn set_service_discovered_callback(
        &mut self,
        service_discovered_callback: Box<ServiceDiscoveredCallback>,
    ) {
        unsafe { (*self.context).service_discovered_callback = Some(service_discovered_callback) };
    }

    /// Sets the optional user context to pass through to the callback. This is useful if you need
    /// to share state between pre and post-callback. The context type must implement `Any`.
    pub fn set_context(&mut self, context: Box<dyn Any>) {
        unsafe { (*self.context).user_context = Some(Arc::from(context)) };
    }

    /// Starts the browser; continuously polling the event loop. This call will block the current
    /// thread.
    pub fn start(&mut self) -> Result<()> {
        debug!("Browsing services: {:?}", self);

        self.poll = Some(ManagedAvahiSimplePoll::new()?);

        let client = ManagedAvahiClient::new(
            ManagedAvahiClientParams::builder()
                .poll(self.poll.as_ref().unwrap())
                .flags(AvahiClientFlags(0))
                .callback(Some(client_callback))
                .userdata(ptr::null_mut())
                .build()?,
        )?;

        unsafe {
            (*self.context).client = Some(client);

            self.browser = Some(ManagedAvahiServiceBrowser::new(
                ManagedAvahiServiceBrowserParams::builder()
                    .client(&(*self.context).client.as_ref().unwrap())
                    .interface(self.interface_index)
                    .protocol(constants::AVAHI_PROTO_UNSPEC)
                    .kind(self.kind.as_ptr())
                    .domain(ptr::null_mut())
                    .flags(0)
                    .callback(Some(browse_callback))
                    .userdata(self.context as *mut c_void)
                    .build()?,
            )?);
        }

        self.poll.as_ref().unwrap().start_loop()
    }
}

impl Drop for AvahiMdnsBrowser {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.context);
        }
    }
}

#[derive(FromRaw, AsRaw)]
struct AvahiBrowserContext {
    client: Option<ManagedAvahiClient>,
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
            .aprotocol(constants::AVAHI_PROTO_UNSPEC)
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
    _txt: *mut AvahiStringList,
    _flags: AvahiLookupResultFlags,
    userdata: *mut c_void,
) {
    let name = cstr::raw_to_str(name);
    let kind = cstr::raw_to_str(kind);
    let domain = cstr::raw_to_str(domain);

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
                cstr::raw_to_str(host_name),
                addr,
                name,
                kind,
                domain,
                port,
            );

            if let Err(e) = result {
                context.invoke_callback(Err(e));
            }
        }
        _ => {}
    };

    context.resolvers.remove_raw(resolver);
}

unsafe fn handle_resolver_found(
    context: &AvahiBrowserContext,
    host_name: &str,
    addr: *const AvahiAddress,
    name: &str,
    kind: &str,
    domain: &str,
    port: u16,
) -> Result<()> {
    let address = avahi_util::avahi_address_to_string(addr);

    let result = ServiceDiscovery::builder()
        .name(name.to_string())
        .kind(kind.to_string())
        .domain(domain.to_string())
        .host_name(host_name.to_string())
        .address(address)
        .port(port)
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
