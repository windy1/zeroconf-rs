use super::avahi_util;
use avahi_sys::{
    avahi_simple_poll_free, avahi_simple_poll_loop, avahi_simple_poll_new, AvahiSimplePoll,
};
use std::ptr;

#[derive(Debug)]
pub struct ManagedAvahiSimplePoll {
    pub(super) poll: *mut AvahiSimplePoll,
}

impl ManagedAvahiSimplePoll {
    pub fn new() -> Result<Self, String> {
        let poll = unsafe { avahi_simple_poll_new() };
        if poll == ptr::null_mut() {
            Err("could not initialize AvahiSimplePoll".to_string())
        } else {
            Ok(Self { poll })
        }
    }

    pub fn start_loop(&self) -> Result<(), String> {
        let err = unsafe { avahi_simple_poll_loop(self.poll) };
        if err != 0 {
            Err(format!(
                "could not start AvahiSimplePoll: {}",
                avahi_util::get_error(err)
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for ManagedAvahiSimplePoll {
    fn drop(&mut self) {
        unsafe { avahi_simple_poll_free(self.poll) };
    }
}
