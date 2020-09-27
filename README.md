# zeroconf

`zeroconf` is a cross-platform library that wraps underlying [ZeroConf/mDNS] implementations
such as [Bonjour] or [Avahi], providing an easy and idiomatic way to both register and
browse services.

## Prerequisites

On Linux:

```bash
$ sudo apt install xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev clang
```

## TODO

* TXT Record support
* Windows support
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
use zeroconf::{MdnsService, ServiceRegistration};

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

fn main() {
    let mut service = MdnsService::new("_http._tcp", 8080);
    let context: Arc<Mutex<Context>> = Arc::default();

    service.set_registered_callback(Box::new(on_service_registered));
    service.set_context(Box::new(context));

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
use zeroconf::{MdnsBrowser, ServiceDiscovery};

fn main() {
    let mut browser = MdnsBrowser::new("_http._tcp");

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

[ZeroConf/mDNS]: https://en.wikipedia.org/wiki/Zero-configuration_networking
[Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
[Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)
[`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
