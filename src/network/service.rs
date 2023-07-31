use super::types::WsMessage;
use super::{net_addr::NetAddress, session::SessionHandle, types::SessionMap};
use log::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Weak};
use tokio::{net::TcpListener, sync::mpsc};

pub enum ServiceMessage {
    SetHandleRef { ptr: Arc<ServiceHandle> },
    StartService,
    Broadcast { msg: WsMessage, source: SocketAddr },
}

pub struct Service {
    receiver: mpsc::UnboundedReceiver<ServiceMessage>,
    net_addr: NetAddress,
    sessions: SessionMap,
    listener: Option<TcpListener>,
    handle_ref: Option<Arc<ServiceHandle>>,
}

impl Service {
    // Construct reference counted Service
    pub fn new(net_addr: NetAddress, receiver: mpsc::UnboundedReceiver<ServiceMessage>) -> Self {
        Self {
            receiver,
            net_addr,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            listener: None,
            handle_ref: None,
        }
    }
    //TODO : channel closed issue
    pub async fn handle_message(&mut self, msg: ServiceMessage) {
        match msg {
            ServiceMessage::SetHandleRef { ptr } => self.handle_ref = Some(ptr),
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

                    let handle_ref = Arc::clone(self.handle_ref.as_ref().unwrap());
                    let session_handle = SessionHandle::new(addr, ws_stream, handle_ref);
                    session_handle.register_recv();
                    self.sessions.lock().unwrap().insert(addr, session_handle);
                }
            }
            ServiceMessage::Broadcast { msg, source } => {
                let sessions = self.sessions.lock().unwrap();
                let broadcast_recipients = sessions
                    .iter()
                    .filter(|(addr, _)| addr != &&source)
                    .map(|(_, handle)| handle);

                for recp in broadcast_recipients {
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
    sender: mpsc::UnboundedSender<ServiceMessage>,
    me: Weak<ServiceHandle>,
}

impl ServiceHandle {
    pub fn new() -> Arc<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = Service::new(NetAddress::new(127, 0, 0, 1, 8080), receiver);
        tokio::spawn(run_service_actor(actor));

        Arc::new_cyclic(|me| ServiceHandle {
            sender,
            me: me.clone(),
        })
    }

    pub fn get_handle_ref(&self) -> Arc<Self> {
        self.me.upgrade().unwrap()
    }

    pub async fn send_handle_ref(&self) {
        let ptr = self.get_handle_ref();
        let msg = ServiceMessage::SetHandleRef { ptr };

        if let Err(e) = self.sender.send(msg) {
            error!("reasion: {} | while: send handle ref to service actor", e);
        }
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

        match self.sender.send(msg) {
            Ok(_) => {
                info!("broadcast received message to other clients")
            }
            Err(e) => {
                error!("reason: {} | while try broacasting to clients", e)
            }
        }
    }
    // pub fn get_handle_ref() -> Arc<Self> {

    // }
}

async fn run_service_actor(mut actor: Service) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await
    }
}
