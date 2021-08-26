use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceType};

#[tokio::main]
pub async fn main() -> zeroconf::Result<()> {
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp")?);
    loop {
        let result = browser.browse_async().await;
        println!("Service discovered: {:?}", result.unwrap());
    }
}
