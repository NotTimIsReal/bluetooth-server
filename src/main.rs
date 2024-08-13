//! Discover Bluetooth devices and list them.

use bluer::{
    agent::{Agent, ReqResult, RequestPasskey, RequestPinCode},
    rfcomm::Profile,
    DiscoveryFilter, DiscoveryTransport,
};
mod bluetooth;
mod server;

async fn request_pin_code(_req: RequestPinCode) -> ReqResult<String> {
    Ok("9999".into())
}
async fn request_passkey(_req: RequestPasskey) -> ReqResult<u32> {
    Ok(9999)
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    env_logger::init();
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Auto,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
    let mut device = bluetooth::Device::new("CLOCK".to_string(), adapter);
    device.search_device_return_addr("CLOCK").await?;
    let agent = Agent {
        request_pin_code: Some(Box::new(|req| Box::pin(request_pin_code(req)))),
        request_passkey: Some(Box::new(|req| Box::pin(request_passkey(req)))),
        ..Default::default()
    };
    let _ahandle = match session.register_agent(agent).await {
        Ok(handle) => handle,
        Err(e) => {
            println!("Agent Registration Failed: {:?}", e);
            return Ok(());
        }
    };
    let mut server = server::server::new(device);
    server.start().await.unwrap();
    server.run().await.unwrap();
    Ok(())
}
