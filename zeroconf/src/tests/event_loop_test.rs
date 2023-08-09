use crate::prelude::*;
use crate::{MdnsService, ServiceType, TxtRecord};
use std::time::{Duration, Instant};

const TEST_DURATION: Duration = Duration::from_secs(1);

const FAST_SPIN_TIMEOUT: Duration = Duration::from_secs(0);
const FAST_SPIN_MIN_ITERS: u32 = 10_000;

const LONG_POLL_TIMEOUT: Duration = Duration::from_secs(1);
const LONG_POLL_MAX_ITERS: u32 = 100;

#[test]
fn event_loop_spins_fast() {
    super::setup();

    static SERVICE_NAME: &str = "event_loop_test_service";
    let mut service = MdnsService::new(ServiceType::new("http", "tcp").unwrap(), 8080);

    let mut txt = TxtRecord::new();
    txt.insert("foo", "bar").unwrap();

    service.set_name(SERVICE_NAME);
    service.set_txt_record(txt.clone());
    service.set_registered_callback(Box::new(|_, _| {
        debug!("Service published");
    }));

    let start = Instant::now();
    let mut iterations = 0;
    let event_loop = service.register().unwrap();
    loop {
        event_loop.poll(FAST_SPIN_TIMEOUT).unwrap();

        if Instant::now().saturating_duration_since(start) >= TEST_DURATION {
            break;
        }
        iterations += 1;
    }

    println!(
        "service loop spun {} times in {} sec",
        iterations,
        TEST_DURATION.as_secs()
    );

    assert!(iterations > FAST_SPIN_MIN_ITERS);
}

#[test]
fn event_loop_long_polls() {
    super::setup();

    static SERVICE_NAME: &str = "event_loop_test_service";
    let mut service = MdnsService::new(ServiceType::new("http", "tcp").unwrap(), 8080);

    let mut txt = TxtRecord::new();
    txt.insert("foo", "bar").unwrap();

    service.set_name(SERVICE_NAME);
    service.set_txt_record(txt.clone());
    service.set_registered_callback(Box::new(|_, _| {
        debug!("Service published");
    }));

    let start = Instant::now();
    let mut iterations = 0;
    let event_loop = service.register().unwrap();
    loop {
        event_loop.poll(LONG_POLL_TIMEOUT).unwrap();

        if Instant::now().saturating_duration_since(start) >= TEST_DURATION {
            break;
        }
        iterations += 1;
    }

    println!(
        "service loop spun {} times in {} sec",
        iterations,
        TEST_DURATION.as_secs()
    );

    assert!(LONG_POLL_MAX_ITERS > iterations);
}
