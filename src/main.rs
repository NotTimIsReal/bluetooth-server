//! Discover Bluetooth devices and list them.

use bluer::{
    agent::{Agent, ReqResult, RequestPasskey, RequestPinCode},
    rfcomm::Profile,
    DiscoveryFilter, DiscoveryTransport,
};
mod bluetooth;

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
    let _ahandle = session.register_agent(agent).await?;
    device.pair().await?;
    // let profile_handle = session.register_profile(Profile::default()).await?;
    // profile_handle
    device.start_comm().await?;
    Ok(())
}
