#[macro_use]
extern crate log;

use clap::Parser;

use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{BrowserEvent, MdnsBrowser, ServiceType};

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
    env_logger::Builder::from_env(env_logger::Env::new().filter_or("RUST_LOG", "info")).init();

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

    browser.set_service_callback(Box::new(on_service_discovery_event));

    let event_loop = browser.browse_services()?;

    loop {
        // calling `poll()` will keep this browser alive
        event_loop.poll(Duration::from_secs(0))?;
    }
}

fn on_service_discovery_event(
    result: zeroconf::Result<BrowserEvent>,
    _context: Option<Arc<dyn Any>>,
) {
    info!(
        "Service event: {:?}",
        result.expect("service discovery failed")
    );

    // ...
}
