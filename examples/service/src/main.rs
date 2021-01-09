use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zeroconf::{MdnsService, ServiceRegistration, TxtRecord};
use zeroconf::prelude::*;

// #[derive(Default, Debug)]
// pub struct Context {
//     service_name: String,
// }
//
// fn main() {
//     let mut service = MdnsService::new("_http._tcp", 8080);
//     let mut txt_record = TxtRecord::new();
//     let context: Arc<Mutex<Context>> = Arc::default();
//
//     txt_record.insert("foo", "bar").unwrap();
//
//     service.set_registered_callback(Box::new(on_service_registered));
//     service.set_context(Box::new(context));
//     service.set_txt_record(txt_record);
//
//     let event_loop = service.register().unwrap();
//
//     loop {
//         // calling `poll()` will keep this service alive
//         event_loop.poll(Duration::from_secs(0)).unwrap();
//     }
// }
//
// fn on_service_registered(
//     result: zeroconf::Result<ServiceRegistration>,
//     context: Option<Arc<dyn Any>>,
// ) {
//     let service = result.unwrap();
//
//     println!("Service registered: {:?}", service);
//
//     let context = context
//         .as_ref()
//         .unwrap()
//         .downcast_ref::<Arc<Mutex<Context>>>()
//         .unwrap()
//         .clone();
//
//     context.lock().unwrap().service_name = service.name().clone();
//
//     println!("Context: {:?}", context);
//
//     // ...
// }

// fn create_service(port: u16) -> (Option<MdnsService>, impl Fn() -> ()) {
//     let mut service: AvahiMdnsService = MdnsService::new("_myapp._tcp", port);
//     service.set_registered_callback(Box::new(|_, _| println!("Registered")));
//
//     let event_loop = service.register().unwrap();
//
//     let poll = move || event_loop.poll(Duration::from_secs(0)).unwrap();
//
//     // If you replace the Option with None, "service" is dropped right there and:
//     //   * The callback is not invoked
//     //   * The service is not broadcasted on the network (but no crash/SIGSEGV here for some reason)
//     // Currently, it works because the service is given back to the main function, and thus, lives until the loop breaks
//     (Some(service), poll)
// }
//
// fn main() {
//     let (_service, poll) = create_service(1337);
//
//     loop {
//         poll();
//     }
// }

fn main() {
    let mut service = create_service(1337);
    let event_loop = service.register().unwrap();

    let poll = move || event_loop.poll(Duration::from_secs(0)).unwrap();

    loop {
        poll();
    }
}

fn create_service(port: u16) -> MdnsService {
    let mut service = MdnsService::new("_myapp._tcp", port);
    service.set_registered_callback(Box::new(|_, _| println!("Registered")));
    service
}
