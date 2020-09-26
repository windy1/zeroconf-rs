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

    let event_loop = service.register().unwrap();

    loop {
        // calling `poll()` will keep this service alive
        event_loop.poll().unwrap();
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
