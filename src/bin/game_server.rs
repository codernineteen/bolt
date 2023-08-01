use server_core::network::{global::G_SERVICE_HANDLE, net_addr::NetAddress};
use std::io::Error as IoError;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    // TODO : service initialize
    env_logger::init();

    let net_addr = NetAddress::new(127, 0, 0, 1, 8080);
    let try_listener = TcpListener::bind(net_addr.addr_str.clone()).await;
    let listener = try_listener.expect("Failed to bind addr");

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(G_SERVICE_HANDLE.handle_connection(stream, addr));
    }

    Ok(())
}
