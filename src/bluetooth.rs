use bluer::{
    l2cap::stream,
    rfcomm::{SocketAddr, Stream},
    Adapter, AdapterEvent, Address, DeviceEvent, DiscoveryFilter, DiscoveryTransport,
};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env, fmt::format};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    stream: Option<Stream>,
}

impl Device {
    pub fn new(name: String, adapter: Adapter) -> Self {
        Self {
            name,
            address: Address::default(),
            adapter,
            stream: None,
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
        let err = device.pair().await;
        match err {
            Err(e) => {
                if e.kind == bluer::ErrorKind::AlreadyExists {
                    return Ok(());
                } else {
                    return Err(e);
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub async fn start_comm(&mut self) -> Result<(), bluer::Error> {
        let socket_addr = SocketAddr {
            addr: self.address,
            channel: 1,
        };
        let mut stream = match Stream::connect(socket_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(bluer::Error {
                    kind: bluer::ErrorKind::AlreadyExists,
                    message: format!("Error Connecting Stream {:?}", e),
                });
            }
        };
        self.stream = Some(stream);

        Ok(())
    }
    async fn shutdown(&mut self) -> Result<(), bluer::Error> {
        match self.stream.as_mut() {
            Some(stream) => {
                stream.shutdown().await?;
            }
            None => {
                return Err(bluer::Error {
                    kind: bluer::ErrorKind::AlreadyExists,
                    message: "Stream not found".to_string(),
                });
            }
        }
        Ok(())
    }
    pub async fn send_message(&mut self, content: &[u8]) -> Result<(), bluer::Error> {
        match self.stream.as_mut() {
            Some(stream) => {
                stream.write(content).await?;
            }
            None => {
                return Err(bluer::Error {
                    kind: bluer::ErrorKind::AlreadyExists,
                    message: "Stream not found".to_string(),
                });
            }
        }
        Ok(())
    }
    pub async fn receive_message(&mut self) -> Result<Vec<u8>, bluer::Error> {
        let mut buffer = vec![0; 1024];
        match self.stream.as_mut() {
            Some(stream) => {
                stream.read(&mut buffer).await?;
            }
            None => {
                return Err(bluer::Error {
                    kind: bluer::ErrorKind::AlreadyExists,
                    message: "Stream not found".to_string(),
                });
            }
        }
        Ok(buffer)
    }
}
