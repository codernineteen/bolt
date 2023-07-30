use super::{
    service::ServiceHandle,
    types::{WsMessage, WsMessageReceiver, WsMessageSender, WsRecvBuffer, WsSendBuffer, WsStream},
};
use futures_util::{SinkExt, StreamExt}; // pin_mut, stream::TryStreamExt, StreamExt};
use log::{error, info};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;

trait Session {
    fn on_connect(addr: SocketAddr);
    fn on_disconnect();
    fn on_recv() -> i32;
    fn on_send();
}

pub enum SessionMessage {
    OnSend,
    OnRecv,
}

pub struct SessionHandle {
    sender: mpsc::UnboundedSender<SessionMessage>,
}

pub struct GameSession {
    addr: SocketAddr,
    recv_buffer: WsRecvBuffer,
    send_buffer: WsSendBuffer,
    ws_sender: WsMessageSender,
    ws_receiver: WsMessageReceiver,
    msg_receiver: mpsc::UnboundedReceiver<SessionMessage>,
}

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

impl GameSession {
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
            ws_receiver: rx,
            ws_sender: tx,
            msg_receiver,
        }
    }
    //TODO : channel closed issue
    pub async fn handle_message(&mut self, msg: SessionMessage) {
        match msg {
            SessionMessage::OnSend => {
                while let Some(msg) = self.ws_receiver.recv().await {
                    if let Err(e) = self.send_buffer.send(msg).await {
                        error!(
                            "reason : {} | process : send msg from session to buffer ",
                            e
                        );
                    };
                }
            }
            SessionMessage::OnRecv => {
                while let Some(msg) = self.recv_buffer.next().await {
                    let msg = msg.unwrap();
                    let msg_text = msg.clone().into_text().unwrap();
                    info!("got message from {} : {}", self.addr, msg_text);

                    if let Err(e) = self.ws_sender.send(msg) {
                        error!("reason : {} | process : send msg to ws_receiver", e);
                    }
                }
            }
        }
    }
}

impl SessionHandle {
    pub fn new(addr: SocketAddr, ws_stream: WsStream) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = GameSession::new(addr, ws_stream, receiver);
        tokio::spawn(run_session_actor(actor));

        Self { sender }
    }

    pub fn register_send(&self) {
        let msg = SessionMessage::OnSend;
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

async fn run_session_actor(mut actor: GameSession) {
    while let Some(msg) = actor.msg_receiver.recv().await {
        actor.handle_message(msg).await
    }
}
