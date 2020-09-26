use crate::{MdnsBrowser, MdnsService};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn service_register_is_browsable() {
    super::setup();

    #[derive(Default, Debug)]
    struct Context {
        is_discovered: bool,
    }

    static SERVICE_NAME: &str = "service_register_is_browsable";
    let mut service = MdnsService::new("_http._tcp", 8080);
    let context: Arc<Mutex<Context>> = Arc::default();

    service.set_name(SERVICE_NAME);
    service.set_context(Box::new(context.clone()));

    service.set_registered_callback(Box::new(|_, context| {
        let mut browser = MdnsBrowser::new("_http._tcp");

        let context = context
            .as_ref()
            .unwrap()
            .downcast_ref::<Arc<Mutex<Context>>>()
            .unwrap()
            .clone();

        browser.set_context(Box::new(context.clone()));

        browser.set_service_discovered_callback(Box::new(|service, context| {
            let service = service.unwrap();

            if service.name() == SERVICE_NAME {
                context
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<Arc<Mutex<Context>>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .is_discovered = true;
            }
        }));

        let event_loop = browser.browse_services().unwrap();

        loop {
            event_loop.poll(Duration::from_secs(0)).unwrap();
            if context.lock().unwrap().is_discovered {
                break;
            }
        }
    }));

    let event_loop = service.register().unwrap();

    loop {
        event_loop.poll(Duration::from_secs(0)).unwrap();
        if context.lock().unwrap().is_discovered {
            break;
        }
    }
}
