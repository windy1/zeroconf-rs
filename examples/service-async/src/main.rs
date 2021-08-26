use std::thread;
use std::time::Duration;
use zeroconf::prelude::*;
use zeroconf::{MdnsService, ServiceType, TxtRecord};

#[tokio::main]
pub async fn main() -> zeroconf::Result<()> {
    let mut service = MdnsService::new(ServiceType::new("http", "tcp")?, 8080);
    let mut txt_record = TxtRecord::new();

    txt_record.insert("hello", "world")?;
    service.set_txt_record(txt_record);

    let result = service.register_async().await;
    println!("Service: {:?}", result);

    loop {
        // do stuff
        thread::sleep(Duration::from_nanos(1));
    }
}
