use bluer::{
    rfcomm::{SocketAddr, Stream},
    Adapter, AdapterEvent, Address, DeviceEvent, DiscoveryFilter, DiscoveryTransport,
};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env};
async fn compare_addr_with_name(
    adapter: &Adapter,
    name: &str,
    addr: Address,
) -> bluer::Result<Option<Address>> {
    let device = adapter.device(addr)?;
    let device_name = device.name().await?;
    if device_name == Some(name.into()) {
        return Ok(Some(addr));
    }
    Ok(None)
}

pub struct Device {
    name: String,
    address: Address,
    adapter: Adapter,
}

impl Device {
    pub fn new(name: String, adapter: Adapter) -> Self {
        Self {
            name,
            address: Address::default(),
            adapter,
        }
    }
    pub async fn search_device_return_addr(&mut self, name: &str) -> Result<Address, bluer::Error> {
        let device_events = self.adapter.discover_devices().await?;
        pin_mut!(device_events);
        let mut address: Address = Address::default();
        let mut i = 0;
        loop {
            i += 1;
            if i > 500 {
                break;
            }
            tokio::select! {
                Some(device_event) = device_events.next() => {
                    match device_event {
                        AdapterEvent::DeviceAdded(addr) => {
                         let addr=compare_addr_with_name(&self.adapter, "CLOCK", addr).await;
                         match addr{
                            Ok(Some(addr)) => {

                              address=addr;
                              break;
                            }
                            _ =>
                                continue
                            ,
                         }

                        }
                        _ => continue,
                    }

                }

                else => break
            }
        }
        //check if address is not empty
        if address == Address::default() {
            println!("Address default");
            return Err(bluer::Error {
                kind: bluer::ErrorKind::AlreadyExists,
                message: "Device not found".to_string(),
            });
        }
        self.address = address;
        Ok(address)
    }
    pub async fn pair(&self) -> Result<(), bluer::Error> {
        let device = self.adapter.device(self.address)?;
        device.pair().await?;
        Ok(())
    }
    pub async fn start_comm(&self) -> Result<(), bluer::Error> {
        let socket_addr = SocketAddr {
            addr: self.address,
            channel: 1,
        };
        let stream = Stream::connect(socket_addr)
            .await
            .expect("Connection failed");
        Ok(())
    }
}
