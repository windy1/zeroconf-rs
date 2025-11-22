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
    context: Option<Arc<dyn Any + Send + Sync>>,
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
        .service_name = service.name().to_string();

    info!("Context: {:?}", context);

    // ...
}
