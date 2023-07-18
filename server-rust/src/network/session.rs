use {
    super::service::Service,
    futures_channel::mpsc::{unbounded, UnboundedSender},
    futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt},
    std::{
        collections::HashMap,
        net::SocketAddr,
        sync::{atomic::AtomicBool, Arc, Mutex, Weak},
    },
    tokio::net::TcpStream,
    tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream},
};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub struct Session {
    service: Weak<Service>,
    socket: WebSocketStream<TcpStream>,
    connected: AtomicBool,
}

impl Session {
    pub fn new(service: Weak<Service>, socket: WebSocketStream<TcpStream>) -> Self {
        Self {
            service,
            socket,
            connected: AtomicBool::new(false),
        }
    }

    pub async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
        println!("Incoming TCP connection from: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(raw_stream)
            .await
            .expect("Error during the websocket handshake occurred");
        println!("WebSocket connection established: {}", addr);
    }
}
