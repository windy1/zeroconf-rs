use avahi_sys::{AvahiIfIndex, AvahiProtocol};

pub const AVAHI_IF_UNSPEC: AvahiIfIndex = -1;
pub const AVAHI_PROTO_UNSPEC: AvahiProtocol = -1;
pub const AVAHI_ERR_COLLISION: i32 = -8;
pub const AVAHI_ADDRESS_STR_MAX: usize = 40;
