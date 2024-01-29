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

Bonjour must be installed. It comes bundled with [iTunes][] or [Bonjour Print Services][]. Further redistribution &
bundling details are available on the [Apple Developer Site][].

## Examples

### Register a service

When registering a service, you may optionally pass a "context" to pass state through the
callback. The only requirement is that this context implements the [`Any`] trait, which most
types will automatically. See `MdnsService` for more information about contexts.

```rust
#[macro_use]
extern crate log;

use clap::Parser;

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsService, ServiceRegistration, ServiceType, TxtRecord};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name of the service type to register
    #[clap(short, long, default_value = "http")]
    name: String,

    /// Protocol of the service type to register
    #[clap(short, long, default_value = "tcp")]
    protocol: String,

    /// Sub-types of the service type to register
    #[clap(short, long)]
    sub_types: Vec<String>,
}

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

fn main() -> zeroconf::Result<()> {
    env_logger::init();

    let Args {
        name,
        protocol,
        sub_types,
    } = Args::parse();

    let sub_types = sub_types.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    let service_type = ServiceType::with_sub_types(&name, &protocol, sub_types)?;
    let mut service = MdnsService::new(service_type, 8080);
    let mut txt_record = TxtRecord::new();
    let context: Arc<Mutex<Context>> = Arc::default();

    txt_record.insert("foo", "bar")?;

    service.set_name("zeroconf_example_service");
    service.set_registered_callback(Box::new(on_service_registered));
    service.set_context(Box::new(context));
    service.set_txt_record(txt_record);

    let event_loop = service.register()?;

    loop {
        // calling `poll()` will keep this service alive
        event_loop.poll(Duration::from_secs(0))?;
    }
}

fn on_service_registered(
    result: zeroconf::Result<ServiceRegistration>,
    context: Option<Arc<dyn Any>>,
) {
    let service = result.expect("failed to register service");

    info!("Service registered: {:?}", service);

    let context = context
        .as_ref()
        .expect("could not get context")
        .downcast_ref::<Arc<Mutex<Context>>>()
        .expect("error down-casting context")
        .clone();

    context
        .lock()
        .expect("failed to obtain context lock")
        .service_name = service.name().clone();

    info!("Context: {:?}", context);

    // ...
}
```

### Browsing services

```rust
#[macro_use]
extern crate log;

use clap::Parser;

use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

/// Example of a simple mDNS browser
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name of the service type to browse
    #[clap(short, long, default_value = "http")]
    name: String,

    /// Protocol of the service type to browse
    #[clap(short, long, default_value = "tcp")]
    protocol: String,

    /// Sub-type of the service type to browse
    #[clap(short, long)]
    sub_type: Option<String>,
}

fn main() -> zeroconf::Result<()> {
    env_logger::init();

    let Args {
        name,
        protocol,
        sub_type,
    } = Args::parse();

    let sub_types: Vec<&str> = match sub_type.as_ref() {
        Some(sub_type) => vec![sub_type],
        None => vec![],
    };

    let service_type =
        ServiceType::with_sub_types(&name, &protocol, sub_types).expect("invalid service type");

    let mut browser = MdnsBrowser::new(service_type);

    browser.set_service_discovered_callback(Box::new(on_service_discovered));

    let event_loop = browser.browse_services()?;

    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll(Duration::from_secs(0))?;
    }
}

fn on_service_discovered(
    result: zeroconf::Result<ServiceDiscovery>,
    _context: Option<Arc<dyn Any>>,
) {
    info!(
        "Service discovered: {:?}",
        result.expect("service discovery failed")
    );

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
[iTunes]: https://support.apple.com/en-us/HT210384
[Bonjour Print Services]: https://developer.apple.com/licensing-trademarks/bonjour/
[Apple Developer Site]: https://developer.apple.com/licensing-trademarks/bonjour/
