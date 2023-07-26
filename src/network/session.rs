// use {
//     super::{recv_buffer::RecvBuffer, service::Service},
//     futures_channel::mpsc::{unbounded, UnboundedSender},
//     futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt},
//     std::{
//         collections::HashMap,
//         net::SocketAddr,
//         sync::{atomic::AtomicBool, Arc, Mutex, Weak},
//     },
//     tokio::net::TcpStream,
//     tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream},
// };
use std::net::SocketAddr;

// type replacement
// type Tx = UnboundedSender<Message>;
// type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

// 64KB buffer size default
//const BUFFER_SIZE: usize = 0x10000;

trait Session {
    fn on_connect(addr: SocketAddr);
    fn on_disconnect();
    fn on_recv() -> i32;
    fn on_send();
}

pub struct GameSession {}

impl Session for GameSession {
    fn on_connect(addr: SocketAddr) {
        println!("A user connected to server | port : {}", addr.port());
    }

    fn on_disconnect() {
        println!("A session disconnected");
    }

    fn on_recv() -> i32 {
        // TODO : define the recv logic
        0
    }

    fn on_send() {
        // TODO : define the send logica
    }
}
