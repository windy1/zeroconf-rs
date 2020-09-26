use std::any::Any;
use std::sync::Arc;
use zeroconf::{MdnsBrowser, ServiceDiscovery};

fn main() {
    let mut browser = MdnsBrowser::new("_http._tcp");

    browser.set_service_discovered_callback(Box::new(on_service_discovered));

    let event_loop = browser.browse_services().unwrap();

    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll().unwrap();
    }
}

fn on_service_discovered(
    result: zeroconf::Result<ServiceDiscovery>,
    _context: Option<Arc<dyn Any>>,
) {
    println!("Service discovered: {:?}", result.unwrap());

    // ...
}
