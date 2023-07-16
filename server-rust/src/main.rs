mod net_addr;
mod socket_util;

use {
    crate::{net_addr::NetAddress, socket_util::handle_connection},
    futures_channel::mpsc::UnboundedSender,
    std::{
        collections::HashMap,
        io::Error as IoError,
        net::SocketAddr,
        sync::{Arc, Mutex},
    },
    tokio::net::TcpListener,
    tokio_tungstenite::tungstenite::protocol::Message,
};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let net_address = NetAddress::new(127, 0, 0, 1, 8080);

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&net_address.sock_addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {:?}", net_address.ip);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, sock_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, sock_addr));
    }

    Ok(())
}
