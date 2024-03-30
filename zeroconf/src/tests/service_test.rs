use crate::prelude::*;
use crate::{MdnsBrowser, MdnsService, ServiceType, TxtRecord};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default, Debug)]
struct Context {
    is_discovered: bool,
    timed_out: bool,
    txt: Option<TxtRecord>,
}

#[test]
fn service_register_is_browsable() {
    super::setup();

    const TOTAL_TEST_TIME_S: u64 = 30;
    static SERVICE_NAME: &str = "service_register_is_browsable";

    let mut service = MdnsService::new(
        ServiceType::with_sub_types("http", "tcp", vec!["printer"]).unwrap(),
        8080,
    );

    let context: Arc<Mutex<Context>> = Arc::default();

    let mut txt = TxtRecord::new();
    txt.insert("foo", "bar").unwrap();

    service.set_name(SERVICE_NAME);
    service.set_context(Box::new(context.clone()));
    service.set_txt_record(txt.clone());

    service.set_registered_callback(Box::new(|result, context| {
        assert!(result.is_ok());

        let mut browser =
            MdnsBrowser::new(ServiceType::with_sub_types("http", "tcp", vec!["printer"]).unwrap());

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
                let mut mtx = context
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<Arc<Mutex<Context>>>()
                    .unwrap()
                    .lock()
                    .unwrap();

                mtx.txt = service.txt().clone();
                mtx.is_discovered = true;

                debug!("Service discovered");
            }
        }));

        let event_loop = browser.browse_services().unwrap();
        let browse_start = std::time::Instant::now();

        loop {
            event_loop.poll(Duration::from_secs(0)).unwrap();

            if context.lock().unwrap().is_discovered {
                break;
            }

            if browse_start.elapsed().as_secs() > TOTAL_TEST_TIME_S / 2 {
                context.lock().unwrap().timed_out = true;
                break;
            }
        }
    }));

    let event_loop = service.register().unwrap();
    let publish_start = std::time::Instant::now();

    loop {
        event_loop.poll(Duration::from_secs(0)).unwrap();

        let mut mtx = context.lock().unwrap();

        if mtx.is_discovered {
            assert_eq!(txt, mtx.txt.take().unwrap());
            break;
        }

        if publish_start.elapsed().as_secs() > TOTAL_TEST_TIME_S {
            mtx.timed_out = true;
            break;
        }
    }

    assert!(!context.lock().unwrap().timed_out);
}

#[test]
fn service_register_segfault_on_drop() {
    super::setup();
    let service_type = ServiceType::new("http", "tcp").unwrap();
    let mut service = MdnsService::new(service_type, 8080);
    let service_event_loop = service.register().unwrap();
    drop(service);
    service_event_loop.poll(Duration::from_secs(1)).unwrap();
}
