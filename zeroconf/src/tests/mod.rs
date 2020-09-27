use std::sync::Once;

static INIT: Once = Once::new();

pub(self) fn setup() {
    INIT.call_once(|| env_logger::init());
}

mod service_test;
mod txt_record_test;
