use super::{
    service::ServiceHandle,
    types::{WsMessage, WsMessageReceiver, WsMessageSender, WsRecvBuffer, WsSendBuffer, WsStream},
};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use std::net::SocketAddr;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum SessionMessage {
    OnSend { ws_msg: WsMessage },
    OnRecv,
}

pub struct SessionHandle {
    sender: mpsc::UnboundedSender<SessionMessage>,
}

pub struct Session {
    addr: SocketAddr,
    recv_buffer: WsRecvBuffer,
    send_buffer: WsSendBuffer,
    _ws_sender: WsMessageSender,
    _ws_receiver: WsMessageReceiver,
    msg_receiver: mpsc::UnboundedReceiver<SessionMessage>,
}

impl Session {
    pub fn new(
        addr: SocketAddr,
        ws_stream: WsStream,
        msg_receiver: mpsc::UnboundedReceiver<SessionMessage>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<WsMessage>();
        let (send_buffer, recv_buffer) = ws_stream.split();
        Self {
            addr,
            send_buffer,
            recv_buffer,
            _ws_receiver: rx,
            _ws_sender: tx,
            msg_receiver,
        }
    }
    //TODO : channel closed issue
    pub async fn handle_message(&mut self, msg: SessionMessage) {
        match msg {
            SessionMessage::OnSend { ws_msg } => {
                if let Err(e) = self.send_buffer.send(ws_msg).await {
                    error!("reason: {} | while send message to send_buffer", e);
                };
            }
            SessionMessage::OnRecv => {
                while let Some(msg) = self.recv_buffer.next().await {
                    let msg = msg.unwrap();
                    let msg_text = msg.clone().into_text().unwrap();
                    info!("got message from {} : {}", self.addr, msg_text);
                    ServiceHandle::instance().broadcast(msg, self.addr).await;
                }
            }
        }
    }
}

impl SessionHandle {
    pub fn new(addr: SocketAddr, ws_stream: WsStream) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = Session::new(addr, ws_stream, receiver);
        tokio::spawn(run_session_actor(actor));

        Self { sender }
    }

    pub fn register_send(&self, ws_msg: WsMessage) {
        let msg = SessionMessage::OnSend { ws_msg };
        self.sender
            .send(msg)
            .expect("failed to send message to session actor");
    }

    pub fn register_recv(&self) {
        let msg = SessionMessage::OnRecv;
        self.sender
            .send(msg)
            .expect("Failed to send message to session actor");
    }
}

async fn run_session_actor(mut actor: Session) {
    while let Some(msg) = actor.msg_receiver.recv().await {
        info!("{:?}", msg);
        actor.handle_message(msg).await
    }
}
