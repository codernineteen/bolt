use super::{net_addr::NetAddress, session::SessionHandle, types::SessionMap};
use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use tokio::{net::TcpListener, sync::mpsc};

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
    //TODO : channel closed issue
    pub async fn handle_message(&mut self, msg: ServiceMessage) {
        match msg {
            ServiceMessage::StartService => {
                let try_listener = TcpListener::bind(self.net_addr.addr_str.clone()).await;
                self.listener = Some(try_listener.expect("Failed to bind addr"));

                info!("listener binded to {}", self.net_addr.addr_str);

                while let Ok((stream, addr)) = self.listener.as_ref().unwrap().accept().await {
                    stream
                        .set_nodelay(true)
                        .expect("Failed to set nodelay option to socket"); // turn off Nagle algorithm
                    info!("incoming TCP connection from {}", addr);

                    let ws_stream = tokio_tungstenite::accept_async(stream)
                        .await
                        .expect("Error occurred during the websocket handshake");

                    info!("websocket connection established to {}", addr.port());

                    let session_handle = SessionHandle::new(addr, ws_stream);
                    session_handle.register_recv();
                    self.sessions.lock().unwrap().insert(addr, session_handle);
                }
            }
        }
    }
}

impl Default for ServiceHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = Service::new(NetAddress::new(127, 0, 0, 1, 8080), receiver);
        tokio::spawn(run_service_actor(actor));

        Self { sender }
    }

    pub async fn start_service(&self) {
        let msg = ServiceMessage::StartService;

        match self.sender.send(msg) {
            Ok(_) => {
                info!("started a server service")
            }
            Err(e) => {
                error!("failed to send message. {}", e);
            }
        }
    }

    // TODO : implement broadcasting
    pub async fn broadcast(&self) {}
    // pub fn get_handle_ref() -> Arc<Self> {

    // }
}

async fn run_service_actor(mut actor: Service) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await
    }
}
