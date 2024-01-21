# zeroconf

`zeroconf` is a cross-platform library that wraps underlying [ZeroConf/mDNS] implementations
such as [Bonjour] or [Avahi], providing an easy and idiomatic way to both register and
browse services.

## Prerequisites

On Linux:

```bash
$ sudo apt install xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev clang avahi-daemon libavahi-client-dev
```

On Windows:

Bonjour must be installed. It comes bundled with [iTunes](https://support.apple.com/en-us/HT210384) or [Bonjour Print Services](https://support.apple.com/kb/dl999). Further redistribution & bundling details are available on the [Apple Developer Site](https://developer.apple.com/licensing-trademarks/bonjour/).

## TODO

* You tell me...

# Examples

## Register a service

When registering a service, you may optionally pass a "context" to pass state through the
callback. The only requirement is that this context implements the [`Any`] trait, which most
types will automatically. See `MdnsService` for more information about contexts.

```rust
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsService, ServiceRegistration, ServiceType, TxtRecord};

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

fn main() {
    let mut service = MdnsService::new(ServiceType::new("http", "tcp").unwrap(), 8080);
    let mut txt_record = TxtRecord::new();
    let context: Arc<Mutex<Context>> = Arc::default();

    txt_record.insert("foo", "bar").unwrap();

    service.set_registered_callback(Box::new(on_service_registered));
    service.set_context(Box::new(context));
    service.set_txt_record(txt_record);

    let event_loop = service.register().unwrap();

    loop {
        // calling `poll()` will keep this service alive
        event_loop.poll(Duration::from_secs(0)).unwrap();
    }
}

fn on_service_registered(
    result: zeroconf::Result<ServiceRegistration>,
    context: Option<Arc<dyn Any>>,
) {
    let service = result.unwrap();

    println!("Service registered: {:?}", service);

    let context = context
        .as_ref()
        .unwrap()
        .downcast_ref::<Arc<Mutex<Context>>>()
        .unwrap()
        .clone();

    context.lock().unwrap().service_name = service.name().clone();

    println!("Context: {:?}", context);

    // ...
}
```

## Browsing services

```rust
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

fn main() {
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp").unwrap());

    browser.set_service_discovered_callback(Box::new(on_service_discovered));

    let event_loop = browser.browse_services().unwrap();

    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll(Duration::from_secs(0)).unwrap();
    }
}

fn on_service_discovered(
    result: zeroconf::Result<ServiceDiscovery>,
    _context: Option<Arc<dyn Any>>,
) {
    println!("Service discovered: {:?}", result.unwrap());

    // ...
}
```

## Resources

* [Avahi docs]
* [Bonjour docs]

[ZeroConf/mDNS]: https://en.wikipedia.org/wiki/Zero-configuration_networking
[Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
[Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)
[`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
[Avahi docs]: https://avahi.org/doxygen/html/
[Bonjour docs]: https://developer.apple.com/documentation/dnssd/dns_service_discovery_c
