use crate::prelude::*;
use crate::{MdnsBrowser, MdnsService, ServiceType, TxtRecord};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::test]
#[serial_test::serial]
async fn service_register_is_browsable_async() {
    super::setup();

    static SERVICE_NAME: &str = "service_register_is_browsable_async";
    let mut service = MdnsService::new(ServiceType::new("http", "tcp").unwrap(), 8080);
    service.set_name(SERVICE_NAME);

    let result = service.register_async().await.unwrap();
    assert_eq!(result.name(), SERVICE_NAME);

    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp").unwrap());
    let browse_result = browser.browse_async().await.unwrap();
    assert_eq!(browse_result.name(), result.name());
}

#[test]
#[serial_test::serial]
fn service_register_is_browsable() {
    super::setup();

    #[derive(Default, Debug)]
    struct Context {
        is_discovered: bool,
        txt: Option<TxtRecord>,
    }

    static SERVICE_NAME: &str = "service_register_is_browsable";
    let mut service = MdnsService::new(ServiceType::new("http", "tcp").unwrap(), 8080);
    let context: Arc<Mutex<Context>> = Arc::default();

    let mut txt = TxtRecord::new();
    txt.insert("foo", "bar").unwrap();

    service.set_name(SERVICE_NAME);
    service.set_context(Box::new(context.clone()));
    service.set_txt_record(txt.clone());

    service.set_registered_callback(Box::new(|_, context| {
        let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp").unwrap());

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

        let event_loop = browser.browse().unwrap();

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

        let mut mtx = context.lock().unwrap();
        if mtx.is_discovered {
            assert_eq!(txt, mtx.txt.take().unwrap());
            break;
        }
    }
}
