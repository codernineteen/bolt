use super::{
    session::SessionHandle,
    types::{SessionMap, WsMessage},
};
use lazy_static::lazy_static;
use log::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{mpsc, RwLock},
};

#[derive(Debug)]
pub enum ServiceMessage {
    Connect(TcpStream, SocketAddr),
    Broadcast { msg: WsMessage, source: SocketAddr },
}

/**
 * --------------
 * Service Struct
 * --------------
 */

pub struct Service {
    sessions: SessionMap,
    msg_receiver: mpsc::UnboundedReceiver<ServiceMessage>,
    connection_receiver: mpsc::Receiver<ServiceMessage>,
}

impl Service {
    // Construct reference counted Service
    pub fn new(
        msg_receiver: mpsc::UnboundedReceiver<ServiceMessage>,
        connection_receiver: mpsc::Receiver<ServiceMessage>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            msg_receiver,
            connection_receiver,
        }
    }

    pub async fn connect(&mut self, msg: ServiceMessage) {
        match msg {
            ServiceMessage::Connect(stream, addr) => {
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
            _ => error!("This message is not for connection"),
        }
    }

    pub async fn handle_message(&mut self, msg: ServiceMessage) {
        match msg {
            ServiceMessage::Broadcast { msg, source } => {
                let sessions = self.sessions.read().await;

                let broadcast_recipients = sessions
                    .iter()
                    .filter(|(addr, _)| addr != &&source)
                    .map(|(_, handle)| handle);
                for recp in broadcast_recipients {
                    recp.register_send(msg.clone());
                }
            }
            _ => info!("unsupported message"),
        }
    }
}

/**
 * ---------------------
 * Service Handle Struct
 * ---------------------
 */

pub struct ServiceHandle {
    pub msg_sender: mpsc::UnboundedSender<ServiceMessage>,
    pub connection_sender: mpsc::Sender<ServiceMessage>,
}

impl Default for ServiceHandle {
    fn default() -> Self {
        ServiceHandle::new()
    }
}

// Global Arc
// lazy_static! {
//     pub static ref G_SERVICE_HANDLE: Arc<ServiceHandle> = Arc::new(ServiceHandle::default());
// }

impl ServiceHandle {
    pub fn new() -> Self {
        let (msg_sender, msg_receiver) = mpsc::unbounded_channel();
        let (connection_sender, connection_receiver) = mpsc::channel(1);
        let actor = Service::new(msg_receiver, connection_receiver);
        tokio::spawn(run_service_actor(actor));

        Self {
            msg_sender,
            connection_sender,
        }
    }

    // pub fn instance() -> &'static Arc<ServiceHandle> {
    //     &G_SERVICE_HANDLE
    // }

    pub async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let msg = ServiceMessage::Connect(stream, addr);

        if let Err(e) = self.connection_sender.send(msg).await {
            error!("reason: {} | while: send 'StartService' message ", e);
        }
    }

    // TODO : implement broadcasting
    pub async fn broadcast(&self, msg: WsMessage, source: SocketAddr) {
        let msg = ServiceMessage::Broadcast { msg, source };

        if let Err(e) = self.msg_sender.send(msg) {
            error!("reason: {} | while: broadcasting to actor", e);
        }
        info!("sent message to actor");
    }
}

async fn run_service_actor(mut actor: Service) {
    loop {
        tokio::select! {
            Some(msg) = actor.connection_receiver.recv() => {
                info!("Service Handle : {:?}", msg);
                actor.connect(msg).await
            },
            Some(msg) = actor.msg_receiver.recv() => {
                info!("Service Handle : {:?}", msg);
                actor.handle_message(msg).await
            },
            else => break,
        }
    }
}
