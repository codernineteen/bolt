use super::{
    net_addr::NetAddress,
    session::SessionHandle,
    types::{SessionMap, WsMessage},
};
use lazy_static::lazy_static;
use log::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{mpsc, RwLock},
};

#[derive(Debug)]
pub enum ServiceMessage {
    StartService,
    Broadcast { msg: WsMessage, source: SocketAddr },
}

pub struct Service {
    receiver: mpsc::UnboundedReceiver<ServiceMessage>,
    net_addr: NetAddress,
    sessions: SessionMap,
    listener: Option<TcpListener>,
}

impl Service {
    // Construct reference counted Service
    pub fn new(net_addr: NetAddress, receiver: mpsc::UnboundedReceiver<ServiceMessage>) -> Self {
        Self {
            receiver,
            net_addr,
            sessions: Arc::new(RwLock::new(HashMap::new())),
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
                    self.sessions.write().await.insert(addr, session_handle);
                }
            }
            ServiceMessage::Broadcast { msg, source } => {
                println!("messsage incoming");
                let sessions = self.sessions.read().await;
                //println!("{:?}", sessions.keys());
                let broadcast_recipients = sessions
                    .iter()
                    .filter(|(addr, _)| addr != &&source)
                    .map(|(_, handle)| handle);
                for recp in broadcast_recipients {
                    info!("there is session handle");
                    recp.register_send(msg.clone());
                }
            }
        }
    }
}

/**
 * ServiceHandle
 */

#[derive(Clone)]
pub struct ServiceHandle {
    pub sender: mpsc::UnboundedSender<ServiceMessage>,
}

impl Default for ServiceHandle {
    fn default() -> Self {
        ServiceHandle::new()
    }
}

lazy_static! {
    pub static ref G_SERVICE_HANDLE: Arc<ServiceHandle> = Arc::new(ServiceHandle::default());
}

impl ServiceHandle {
    pub fn new() -> Self {
        println!("new service handle");
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = Service::new(NetAddress::new(127, 0, 0, 1, 8080), receiver);
        tokio::spawn(run_service_actor(actor));

        Self { sender }
    }

    pub fn instance() -> &'static Arc<ServiceHandle> {
        &G_SERVICE_HANDLE
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
    pub async fn broadcast(&self, msg: WsMessage, source: SocketAddr) {
        let msg = ServiceMessage::Broadcast { msg, source };

        if let Err(e) = self.sender.send(msg) {
            error!("reason: {} | while: broadcasting to actor", e);
        }
        info!("sent message to actor");
    }
}

async fn run_service_actor(mut actor: Service) {
    loop {
        match actor.receiver.recv().await {
            Some(msg) => {
                info!("{:?}", msg);
                actor.handle_message(msg).await;
            }
            None => println!("sender closed"),
        }
    }
}
