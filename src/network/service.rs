use super::{net_addr::NetAddress, session::GameSession};
use anyhow::Error;
use std::collections::HashMap;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, mpsc::error::SendError, oneshot},
};

type SessionMap = Arc<Mutex<HashMap<SocketAddr, GameSession>>>;

pub enum ServiceMessage {
    StartService,
}

pub struct Service {
    receiver: mpsc::UnboundedReceiver<ServiceMessage>,
    net_addr: NetAddress,
    sessions: SessionMap,
    listener: Option<TcpListener>,
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
            listener: None,
        }
    }

    pub async fn handle_message(&mut self, msg: ServiceMessage) {
        match msg {
            ServiceMessage::StartService => {
                let try_listener = TcpListener::bind(self.net_addr.addr_str.clone()).await;
                self.listener = Some(try_listener.expect("Failed to bind addr"));

                println!("Listener binded to {}", self.net_addr.addr_str);

                loop {
                    if let Ok((stream, addr)) = self.listener.as_ref().unwrap().accept().await {
                        stream
                            .set_nodelay(true)
                            .expect("Failed to set nodelay option to socket"); // turn off Nagle algorithm
                        tokio::spawn(Service::handle_connection(stream, addr));
                    }
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

impl Default for ServiceHandle {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = Service::new(NetAddress::new(127, 0, 0, 1, 8080), receiver);
        tokio::spawn(run_service_actor(actor));

        Self { sender }
    }
}

impl ServiceHandle {
    pub async fn start_service(&self) -> Result<bool, SendError<ServiceMessage>> {
        let msg = ServiceMessage::StartService;

        //println!("Start service");

        match self.sender.send(msg) {
            Ok(_) => {
                //println!("sent message");
                Ok(true)
            }
            Err(e) => {
                println!("Failed to send message");
                Err(e)
            }
        }
    }
}

async fn run_service_actor(mut actor: Service) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await
    }
}
