# zeroconf

`zeroconf` is a cross-platform library that wraps underlying [ZeroConf/mDNS] implementations
such as [Bonjour] or [Avahi], providing an easy and idiomatic way to both register and
browse services.

## Prerequisites

```bash
$ sudo apt install xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev clang
```

## Examples

 ## Register a service

 When registering a service, you may optionally pass a "context" to pass state through the
 callback. The only requirement is that this context implements the [`Any`] trait, which most
 types will automatically. See [`MdnsService`] for more information about contexts.

```rust
use std::any::Any;
use std::sync::{Arc, Mutex};
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

    // blocks current thread, must keep-alive to keep service active
    service.start().unwrap();
}

fn on_service_registered(service: ServiceRegistration, context: Option<Arc<dyn Any>>) {
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
use zeroconf::{MdnsBrowser, ServiceDiscovery};

fn main() {
    let mut browser = MdnsBrowser::new("_http._tcp");

    browser.set_service_discovered_callback(Box::new(on_service_discovered));

    // blocks current thread, must keep-alive to keep browser active
    browser.start().unwrap()
}

fn on_service_discovered(service: ServiceDiscovery, _context: Option<Arc<dyn Any>>) {
    println!("Service discovered: {:?}", &service);

    // ...
}
```

## TODO

## Caveats

[ZeroConf/mDNS]: https://en.wikipedia.org/wiki/Zero-configuration_networking
[Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
[Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)
[`MdnsService`]: struct.MdnsService.html
[`MdnsBrowser`]: struct.MdnsBrowser.html
[`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
