use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

#[tokio::main]
pub async fn main() -> zeroconf::Result<()> {
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp")?);
    loop {
        let result = browser.browse_async().await;
        println!("Service discovered: {:?}", result.unwrap());
    }
}
