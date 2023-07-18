use {
    server_core::network::{net_addr::NetAddress, service::Service},
    std::io::Error as IoError,
};

#[tokio::main]
async fn main() -> Result<(), IoError> {
    // TODO : service initialize
    let server_service = Service::new(NetAddress::new(127, 0, 0, 1, 8080), 1000);
    server_service.start_service().await?;

    Ok(())
}
