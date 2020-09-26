use crate::{MdnsBrowser, MdnsService};
use bonjour_sys::DNSServiceRefSockFD;
use std::ptr;
use std::sync::{Arc, Mutex};

#[test]
fn service_register_is_browsable() {
    super::setup();

    #[derive(Default, Debug)]
    struct Context {
        is_discovered: bool,
    }

    static service_name: &str = "service_register_is_browsable";
    let mut service = MdnsService::new("_http._tcp", 8080);
    let context: Arc<Mutex<Context>> = Arc::default();

    service.set_name(service_name);
    service.set_context(Box::new(context.clone()));

    service.set_registered_callback(Box::new(|_, context| {
        let mut browser = MdnsBrowser::new("_http._tcp");

        let context = context
            .as_ref()
            .unwrap()
            .downcast_ref::<Arc<Mutex<Context>>>()
            .unwrap()
            .clone();

        browser.set_context(Box::new(context));

        browser.set_service_discovered_callback(Box::new(|service, context| {
            let service = service.unwrap();

            if service.name() == service_name {
                debug!("Discovered");
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

        // loop {
        event_loop.poll().unwrap();
        // }
    }));

    let event_loop = service.register().unwrap();

    loop {
        debug!("loop start");

        let select_result = unsafe {
            let mut sockfd = DNSServiceRefSockFD(service.service.lock().unwrap().service);

            let mut fd_set: ::libc::fd_set = std::mem::zeroed();
            ::libc::FD_ZERO(&mut fd_set);
            ::libc::FD_SET(sockfd, &mut fd_set);

            let mut timeout = ::libc::timeval {
                tv_sec: 1,
                tv_usec: 0,
            };

            select(
                sockfd + 1,
                &mut fd_set,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut timeout,
            )
        };

        debug!("select_result = {:?}", select_result);

        if select_result < 0 {
            panic!("select() reported error");
        }

        if select_result == 0 {
            continue;
        }

        event_loop.poll().unwrap();

        debug!("context_main = {:?}", context);

        if context.lock().unwrap().is_discovered {
            break;
        }

        debug!("loop end");
    }

    debug!("Service was discovered");
}

extern "C" {
    fn select(
        nfds: i32,
        readfds: *mut ::libc::fd_set,
        writefds: *mut ::libc::fd_set,
        exceptfds: *mut ::libc::fd_set,
        timeout: *mut ::libc::timeval,
    ) -> i32;
}
