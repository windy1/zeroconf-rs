/// Represents a network interface for mDNS services
pub enum NetworkInterface {
    /// No interface specified, bind to all available interfaces
    Unspec,
    /// An interface at a specified index
    AtIndex(u32),
}
