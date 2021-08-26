use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

fn main() {
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp").unwrap());

    browser.set_service_discovered_callback(Box::new(on_service_discovered));

    let event_loop = browser.browse().unwrap();

    loop {
        // calling `poll()` will cause the browser to continue discovering services
        event_loop.poll(Duration::from_secs(0)).unwrap();
    }
}

fn on_service_discovered(
    result: zeroconf::Result<ServiceDiscovery>,
    _context: Option<Arc<dyn Any>>,
) {
    println!("Service discovered: {:?}", result.unwrap());

    // do stuff
}
