use std::sync::Once;

static INIT: Once = Once::new();

pub(crate) fn setup() {
    INIT.call_once(env_logger::init);
}

mod event_loop_test;
mod service_test;
