use super::{net_addr::NetAddress, session::GameSession};
use anyhow::Error;
use std::collections::HashMap;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

type SessionMap = Arc<Mutex<HashMap<SocketAddr, GameSession>>>;

pub enum ServiceMessage {
    StartService,
}

pub struct Service {
    receiver: mpsc::UnboundedReceiver<ServiceMessage>,
    net_addr: NetAddress,
    sessions: SessionMap,
    //listener: Option<TcpListener>,
}

#[derive(Clone)]
pub struct ServiceHandle {
    sender: mpsc::UnboundedSender<ServiceMessage>,
}

impl Service {
    // Construct reference counted Service
    pub fn new(net_addr: NetAddress, receiver: mpsc::UnboundedReceiver<ServiceMessage>) -> Self {
        Self {
            receiver,
            net_addr,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            // listener: None,
        }
    }

    pub async fn handle_message(&mut self, msg: ServiceMessage) {
        match msg {
            ServiceMessage::StartService => {
                let try_listener = TcpListener::bind(self.net_addr.addr_str.clone()).await;
                let listener = try_listener.expect("Failed to bind addr");

                println!("Listener binded to {}", self.net_addr.addr_str);

                while let Ok((stream, addr)) = listener.accept().await {
                    stream
                        .set_nodelay(true)
                        .expect("Failed to set nodelay option to socket"); // turn off Nagle algorithm
                    tokio::spawn(Service::handle_connection(stream, addr));
                }
            }
        }
    }

    pub async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
        println!("Incoming TCP connection from: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");
        println!("WebSocket connection established: {}", addr);
    }
}

// impl Default for ServiceHandle {
//     async fn default() -> Self {
//         let (sender, receiver) = mpsc::unbounded_channel();
//         let actor = Service::new(NetAddress::new(127, 0, 0, 1, 8080), receiver);
//         run_service_actor(actor).await;

//         Self { sender }
//     }
// }

impl ServiceHandle {
    pub async fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = Service::new(NetAddress::new(127, 0, 0, 1, 8080), receiver);
        tokio::spawn(run_service_actor(actor));

        Self { sender }
    }

    pub async fn start_service(&self) {
        let msg = ServiceMessage::StartService;

        match self.sender.send(msg) {
            Ok(_) => {
                println!("Started a service")
            }
            Err(e) => {
                println!("Failed to send message. {}", e);
            }
        }
    }
}

async fn run_service_actor(mut actor: Service) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await
    }
}
